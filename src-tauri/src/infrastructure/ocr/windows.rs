use std::path::Path;
use std::process::Command;

use serde::Deserialize;

use super::engine::OcrEngine;
use crate::domain::error::AppError;
use crate::domain::ocr::{BoundingBox, OcrLine, OcrRawResult, OcrWord};

#[derive(Debug, Deserialize)]
struct WinWordJson {
    text: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    #[serde(default)]
    confidence: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct WinLineJson {
    text: String,
    #[serde(default)]
    words: Vec<WinWordJson>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WinOcrOutputJson {
    text: String,
    #[serde(default)]
    text_angle: f64,
    #[serde(default)]
    image_width: Option<u32>,
    #[serde(default)]
    image_height: Option<u32>,
    #[serde(default)]
    lines: Vec<WinLineJson>,
}

/// Implementasi Engine OCR menggunakan Windows 11 Native API (Windows.Media.Ocr)
pub struct WindowsMediaOcrEngine {
    available: bool,
}

impl WindowsMediaOcrEngine {
    pub fn new() -> Self {
        let available = cfg!(target_os = "windows") && Self::check_win_ocr_available();
        Self { available }
    }

    fn check_win_ocr_available() -> bool {
        #[cfg(target_os = "windows")]
        {
            let script = r#"
            [Windows.Media.Ocr.OcrEngine, Windows.Foundation, ContentType = WindowsRuntime] | Out-Null
            $engine = [Windows.Media.Ocr.OcrEngine]::TryCreateFromUserProfileLanguages()
            if ($engine -ne $null) { exit 0 } else { exit 1 }
            "#;
            let output = Command::new("powershell")
                .args(["-NoProfile", "-NonInteractive", "-Command", script])
                .output();
            match output {
                Ok(out) => out.status.success(),
                Err(_) => false,
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            false
        }
    }
}

impl Default for WindowsMediaOcrEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl OcrEngine for WindowsMediaOcrEngine {
    fn engine_name(&self) -> &'static str {
        "windows_media_ocr"
    }

    fn is_available(&self) -> bool {
        self.available
    }

    fn recognize_file(&self, image_path: &Path) -> Result<OcrRawResult, AppError> {
        if !self.available {
            return Err(AppError::service_unavailable(
                "Windows Media OCR tidak tersedia pada lingkungan sistem ini.",
            ));
        }

        if !image_path.exists() {
            return Err(AppError::not_found(format!(
                "Berkas gambar tangkapan layar tidak ditemukan di path: {}",
                image_path.display()
            )));
        }

        let abs_path = std::fs::canonicalize(image_path)
            .map_err(|e| AppError::internal(format!("Gagal memvalidasi path gambar OCR: {e}")))?
            .to_string_lossy()
            .to_string();

        let ps_script = format!(
            r#"
            $ErrorActionPreference = 'Stop'
            Add-Type -AssemblyName System.Runtime.WindowsRuntime
            $asTaskGeneric = ([System.WindowsRuntimeSystemExtensions].GetMethods() | Where-Object {{ $_.Name -eq 'AsTask' -and $_.GetParameters().Count -eq 1 -and $_.GetParameters()[0].ParameterType.Name -eq 'IAsyncOperation`1' }})[0]

            function Await($WinRtTask, $ResultType) {{
                $asTask = $asTaskGeneric.MakeGenericMethod($ResultType)
                $netTask = $asTask.Invoke($null, @($WinRtTask))
                $netTask.Wait(-1) | Out-Null
                $netTask.Result
            }}

            [Windows.Storage.StorageFile, Windows.Storage, ContentType = WindowsRuntime] | Out-Null
            [Windows.Graphics.Imaging.BitmapDecoder, Windows.Graphics.Imaging, ContentType = WindowsRuntime] | Out-Null
            [Windows.Media.Ocr.OcrEngine, Windows.Media.Ocr, ContentType = WindowsRuntime] | Out-Null

            $filePath = $env:SURVEY_COPILOT_OCR_PATH
            $file = Await ([Windows.Storage.StorageFile]::GetFileFromPathAsync($filePath)) ([Windows.Storage.StorageFile])
            $stream = Await ($file.OpenAsync([Windows.Storage.FileAccessMode]::Read)) ([Windows.Storage.Streams.IRandomAccessStream])
            $decoder = Await ([Windows.Graphics.Imaging.BitmapDecoder]::CreateAsync($stream)) ([Windows.Graphics.Imaging.BitmapDecoder])
            $softwareBitmap = Await ($decoder.GetSoftwareBitmapAsync()) ([Windows.Graphics.Imaging.SoftwareBitmap])

            $engine = [Windows.Media.Ocr.OcrEngine]::TryCreateFromUserProfileLanguages()
            if ($engine -eq $null) {{
                Write-Error "Engine OCR tidak dapat diinisialisasi."
                exit 1
            }}

            $ocrResult = Await ($engine.RecognizeAsync($softwareBitmap)) ([Windows.Media.Ocr.OcrResult])

            $linesOut = @()
            foreach ($line in $ocrResult.Lines) {{
                $wordsOut = @()
                foreach ($word in $line.Words) {{
                    $wordsOut += @{{
                        text = $word.Text
                        x = [double]$word.BoundingRect.X
                        y = [double]$word.BoundingRect.Y
                        width = [double]$word.BoundingRect.Width
                        height = [double]$word.BoundingRect.Height
                    }}
                }}
                $linesOut += @{{
                    text = $line.Text
                    words = $wordsOut
                }}
            }}

            $result = @{{
                text = $ocrResult.Text
                textAngle = [double]$ocrResult.TextAngle
                imageWidth = [uint32]$decoder.PixelWidth
                imageHeight = [uint32]$decoder.PixelHeight
                lines = $linesOut
            }}

            $result | ConvertTo-Json -Depth 6 -Compress
            "#
        );

        let output = Command::new("powershell")
            .env("SURVEY_COPILOT_OCR_PATH", &abs_path)
            .args(["-NoProfile", "-NonInteractive", "-Command", &ps_script])
            .output()
            .map_err(|e| {
                AppError::internal(format!("Gagal menjalankan proses Windows Media OCR: {e}"))
            })?;

        if !output.status.success() {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::internal(format!(
                "Windows Media OCR gagal memproses gambar: {err_msg}"
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let parsed: WinOcrOutputJson = serde_json::from_str(stdout.trim()).map_err(|e| {
            AppError::internal(format!(
                "Gagal mengurai struktur output JSON dari Windows Media OCR: {e}"
            ))
        })?;

        let mut lines = Vec::new();
        for line in parsed.lines {
            let mut words = Vec::new();
            let mut min_x = f64::MAX;
            let mut min_y = f64::MAX;
            let mut max_x = 0.0f64;
            let mut max_y = 0.0f64;

            for word in line.words {
                min_x = min_x.min(word.x);
                min_y = min_y.min(word.y);
                max_x = max_x.max(word.x + word.width);
                max_y = max_y.max(word.y + word.height);

                words.push(OcrWord {
                    text: word.text,
                    bounding_box: BoundingBox::new(word.x, word.y, word.width, word.height),
                    // Windows.Media.Ocr does not expose a calibrated numeric word score.
                    // 0.0 is the internal "not provided" sentinel; do not fabricate confidence.
                    confidence: word.confidence.unwrap_or(0.0),
                });
            }

            let line_bbox = if min_x <= max_x && min_y <= max_y {
                BoundingBox::new(min_x, min_y, max_x - min_x, max_y - min_y)
            } else {
                BoundingBox::new(0.0, 0.0, 0.0, 0.0)
            };

            let scored_words: Vec<f32> = words
                .iter()
                .map(|word| word.confidence)
                .filter(|confidence| *confidence > 0.0)
                .collect();
            let line_confidence = if scored_words.is_empty() {
                0.0
            } else {
                scored_words.iter().sum::<f32>() / scored_words.len() as f32
            };

            lines.push(OcrLine {
                text: line.text,
                bounding_box: line_bbox,
                words,
                confidence: line_confidence,
            });
        }

        Ok(OcrRawResult {
            raw_text: parsed.text,
            text_angle: parsed.text_angle,
            lines,
            engine: "windows_media_ocr".to_string(),
            image_width: parsed.image_width.unwrap_or(800),
            image_height: parsed.image_height.unwrap_or(600),
        })
    }
}
