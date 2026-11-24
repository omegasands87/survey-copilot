use crate::domain::error::AppError;

/// Encodes raw RGBA pixel bytes to lossless PNG format.
pub fn encode_rgba_to_png(
    width: u32,
    height: u32,
    rgba_pixels: &[u8],
) -> Result<Vec<u8>, AppError> {
    if width == 0 || height == 0 {
        return Err(AppError::validation(
            "dimensions",
            "Dimensi gambar tidak valid (lebar dan tinggi harus lebih besar dari 0).",
        ));
    }

    let expected_len = (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or_else(|| {
            AppError::validation("dimensions", "Dimensi gambar terlalu besar (overflow).")
        })?;

    if rgba_pixels.len() != expected_len {
        return Err(AppError::validation(
            "rgbaPixels",
            format!(
                "Ukuran buffer piksel RGBA ({}) tidak sesuai dengan dimensi {}x{} (diharapkan {} bytes).",
                rgba_pixels.len(),
                width,
                height,
                expected_len
            ),
        ));
    }

    let mut png_bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut png_bytes, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .map_err(|e| AppError::internal(format!("Gagal menginisialisasi PNG writer: {e}")))?;
        writer.write_image_data(rgba_pixels).map_err(|e| {
            AppError::internal(format!("Gagal mengompresi data piksel gambar ke PNG: {e}"))
        })?;
    }

    Ok(png_bytes)
}

/// Converts binary bytes to standard base64 string without external crate dependencies.
pub fn bytes_to_base64(bytes: &[u8]) -> String {
    const CHARSET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);

    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = if chunk.len() > 1 { chunk[1] } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] } else { 0 };

        out.push(CHARSET[(b0 >> 2) as usize] as char);
        out.push(CHARSET[(((b0 & 3) << 4) | (b1 >> 4)) as usize] as char);
        if chunk.len() > 1 {
            out.push(CHARSET[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(CHARSET[(b2 & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
    }

    out
}

/// Formats PNG bytes as a browser-ready base64 data URL.
pub fn png_to_data_url(png_bytes: &[u8]) -> String {
    format!("data:image/png;base64,{}", bytes_to_base64(png_bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_to_base64() {
        assert_eq!(bytes_to_base64(b""), "");
        assert_eq!(bytes_to_base64(b"f"), "Zg==");
        assert_eq!(bytes_to_base64(b"fo"), "Zm8=");
        assert_eq!(bytes_to_base64(b"foo"), "Zm9v");
        assert_eq!(bytes_to_base64(b"foob"), "Zm9vYg==");
        assert_eq!(bytes_to_base64(b"fooba"), "Zm9vYmE=");
        assert_eq!(bytes_to_base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn test_encode_rgba_to_png() {
        let width = 2;
        let height = 2;
        let red_pixel = [255, 0, 0, 255];
        let mut pixels = Vec::new();
        for _ in 0..4 {
            pixels.extend_from_slice(&red_pixel);
        }

        let png = encode_rgba_to_png(width, height, &pixels).expect("Encoding should succeed");
        assert!(!png.is_empty());
        assert_eq!(&png[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]); // Standard PNG header
    }

    #[test]
    fn test_encode_invalid_dimensions() {
        let err = encode_rgba_to_png(0, 0, &[]).unwrap_err();
        assert!(err.message.contains("Dimensi"));
    }
}
