use super::engine::{RawCapturedImage, ScreenCaptureEngine};
use crate::domain::capture::{CaptureRect, WindowInfo};
use crate::domain::error::AppError;

#[cfg(target_os = "windows")]
#[allow(non_snake_case)]
#[allow(clippy::upper_case_acronyms)]
mod ffi {
    use std::ffi::c_void;

    pub type HWND = *mut c_void;
    pub type HDC = *mut c_void;
    pub type HBITMAP = *mut c_void;
    pub type HGDIOBJ = *mut c_void;
    pub type BOOL = i32;
    pub type DWORD = u32;
    pub type UINT = u32;
    pub type LPARAM = isize;
    pub type HRESULT = i32;

    pub const SRCCOPY: DWORD = 0x00CC0020;
    pub const CAPTUREBLT: DWORD = 0x40000000;
    pub const DIB_RGB_COLORS: UINT = 0;
    pub const BI_RGB: DWORD = 0;
    pub const PW_RENDERFULLCONTENT: UINT = 0x00000002;
    pub const DWMWA_CLOAKED: DWORD = 14;

    pub const SM_XVIRTUALSCREEN: i32 = 76;
    pub const SM_YVIRTUALSCREEN: i32 = 77;
    pub const SM_CXVIRTUALSCREEN: i32 = 78;
    pub const SM_CYVIRTUALSCREEN: i32 = 79;
    pub const SM_CXSCREEN: i32 = 0;
    pub const SM_CYSCREEN: i32 = 1;

    #[repr(C)]
    #[derive(Debug, Default, Clone, Copy)]
    pub struct RECT {
        pub left: i32,
        pub top: i32,
        pub right: i32,
        pub bottom: i32,
    }

    #[repr(C)]
    #[derive(Debug, Default, Clone, Copy)]
    pub struct BITMAPINFOHEADER {
        pub biSize: DWORD,
        pub biWidth: i32,
        pub biHeight: i32,
        pub biPlanes: u16,
        pub biBitCount: u16,
        pub biCompression: DWORD,
        pub biSizeImage: DWORD,
        pub biXPelsPerMeter: i32,
        pub biYPelsPerMeter: i32,
        pub biClrUsed: DWORD,
        pub biClrImportant: DWORD,
    }

    #[repr(C)]
    #[derive(Debug, Default, Clone, Copy)]
    pub struct RGBQUAD {
        pub rgbBlue: u8,
        pub rgbGreen: u8,
        pub rgbRed: u8,
        pub rgbReserved: u8,
    }

    #[repr(C)]
    pub struct BITMAPINFO {
        pub bmiHeader: BITMAPINFOHEADER,
        pub bmiColors: [RGBQUAD; 1],
    }

    pub type WNDENUMPROC = unsafe extern "system" fn(HWND, LPARAM) -> BOOL;

    #[link(name = "user32")]
    extern "system" {
        pub fn GetDC(hWnd: HWND) -> HDC;
        pub fn ReleaseDC(hWnd: HWND, hDC: HDC) -> i32;
        pub fn GetWindowRect(hWnd: HWND, lpRect: *mut RECT) -> BOOL;
        pub fn IsWindow(hWnd: HWND) -> BOOL;
        pub fn IsWindowVisible(hWnd: HWND) -> BOOL;
        pub fn IsIconic(hWnd: HWND) -> BOOL;
        pub fn GetWindowTextLengthW(hWnd: HWND) -> i32;
        pub fn GetWindowTextW(hWnd: HWND, lpString: *mut u16, nMaxCount: i32) -> i32;
        pub fn EnumWindows(lpEnumFunc: WNDENUMPROC, lParam: LPARAM) -> BOOL;
        pub fn PrintWindow(hWnd: HWND, hdcBlt: HDC, nFlags: UINT) -> BOOL;
        pub fn GetSystemMetrics(nIndex: i32) -> i32;
    }

    #[link(name = "gdi32")]
    extern "system" {
        pub fn CreateCompatibleDC(hdc: HDC) -> HDC;
        pub fn DeleteDC(hdc: HDC) -> BOOL;
        pub fn CreateCompatibleBitmap(hdc: HDC, cx: i32, cy: i32) -> HBITMAP;
        pub fn SelectObject(hdc: HDC, h: HGDIOBJ) -> HGDIOBJ;
        pub fn DeleteObject(ho: HGDIOBJ) -> BOOL;
        pub fn BitBlt(
            hdc: HDC,
            x: i32,
            y: i32,
            cx: i32,
            cy: i32,
            hdcSrc: HDC,
            x1: i32,
            y1: i32,
            rop: DWORD,
        ) -> BOOL;
        pub fn GetDIBits(
            hdc: HDC,
            hbm: HBITMAP,
            start: UINT,
            cLines: UINT,
            lpvBits: *mut c_void,
            lpbmi: *mut BITMAPINFO,
            usage: UINT,
        ) -> i32;
    }

    #[link(name = "dwmapi")]
    extern "system" {
        pub fn DwmGetWindowAttribute(
            hwnd: HWND,
            dwAttribute: DWORD,
            pvAttribute: *mut c_void,
            cbAttribute: DWORD,
        ) -> HRESULT;
    }
}

pub struct WindowsScreenCaptureEngine;

impl WindowsScreenCaptureEngine {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WindowsScreenCaptureEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_os = "windows")]
impl ScreenCaptureEngine for WindowsScreenCaptureEngine {
    fn list_windows(&self) -> Result<Vec<WindowInfo>, AppError> {
        use ffi::*;

        let mut windows = Vec::new();

        unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
            if IsWindowVisible(hwnd) == 0 {
                return 1;
            }

            // Check if window is cloaked (e.g. suspended UWP apps or background virtual desktop windows)
            let mut cloaked: DWORD = 0;
            let hr = DwmGetWindowAttribute(
                hwnd,
                DWMWA_CLOAKED,
                &mut cloaked as *mut _ as *mut std::ffi::c_void,
                std::mem::size_of::<DWORD>() as DWORD,
            );
            if hr == 0 && cloaked != 0 {
                return 1;
            }

            let title_len = GetWindowTextLengthW(hwnd);
            if title_len <= 0 {
                return 1;
            }

            let mut title_buf = vec![0u16; (title_len + 1) as usize];
            let read_len = GetWindowTextW(hwnd, title_buf.as_mut_ptr(), title_len + 1);
            if read_len <= 0 {
                return 1;
            }

            let title = String::from_utf16_lossy(&title_buf[..read_len as usize])
                .trim()
                .to_string();
            if title.is_empty() {
                return 1;
            }

            let mut rect = RECT::default();
            if GetWindowRect(hwnd, &mut rect) == 0 {
                return 1;
            }

            let width = (rect.right - rect.left).max(0) as u32;
            let height = (rect.bottom - rect.top).max(0) as u32;

            // Filter out 0-size or tiny invisible helper windows
            if width < 32 || height < 32 {
                return 1;
            }

            let is_minimized = IsIconic(hwnd) != 0;
            let window_id = hwnd as usize as u64;

            let windows_list = &mut *(lparam as *mut Vec<WindowInfo>);
            windows_list.push(WindowInfo {
                id: window_id,
                title,
                app_name: None,
                x: rect.left,
                y: rect.top,
                width,
                height,
                is_minimized,
            });

            1
        }

        unsafe {
            let user_data = &mut windows as *mut Vec<WindowInfo> as LPARAM;
            EnumWindows(enum_proc, user_data);
        }

        Ok(windows)
    }

    fn capture_region(&self, bounds: Option<CaptureRect>) -> Result<RawCapturedImage, AppError> {
        use ffi::*;
        use std::ptr;

        unsafe {
            let (x, y, w, h, label) = if let Some(rect) = bounds {
                if rect.width == 0 || rect.height == 0 {
                    return Err(AppError::validation(
                        "bounds",
                        "Ukuran area tangkapan layar tidak valid: lebar dan tinggi harus lebih dari 0.",
                    ));
                }
                (
                    rect.x,
                    rect.y,
                    rect.width,
                    rect.height,
                    format!(
                        "Area Layar ({}, {}, {}x{})",
                        rect.x, rect.y, rect.width, rect.height
                    ),
                )
            } else {
                // Determine virtual screen bounding box (covers multi-monitor setups)
                let vx = GetSystemMetrics(SM_XVIRTUALSCREEN);
                let vy = GetSystemMetrics(SM_YVIRTUALSCREEN);
                let mut vwidth = GetSystemMetrics(SM_CXVIRTUALSCREEN);
                let mut vheight = GetSystemMetrics(SM_CYVIRTUALSCREEN);

                if vwidth <= 0 || vheight <= 0 {
                    vwidth = GetSystemMetrics(SM_CXSCREEN);
                    vheight = GetSystemMetrics(SM_CYSCREEN);
                }

                let final_w = vwidth.max(1) as u32;
                let final_h = vheight.max(1) as u32;

                (
                    vx,
                    vy,
                    final_w,
                    final_h,
                    "Seluruh Tampilan Layar".to_string(),
                )
            };

            let hdc_screen = GetDC(ptr::null_mut());
            if hdc_screen.is_null() {
                return Err(AppError::internal(
                    "Gagal mendapatkan Device Context (DC) layar Windows.",
                ));
            }

            let hdc_mem = CreateCompatibleDC(hdc_screen);
            if hdc_mem.is_null() {
                ReleaseDC(ptr::null_mut(), hdc_screen);
                return Err(AppError::internal(
                    "Gagal membuat Compatible DC untuk tangkapan layar.",
                ));
            }

            let hbitmap = CreateCompatibleBitmap(hdc_screen, w as i32, h as i32);
            if hbitmap.is_null() {
                DeleteDC(hdc_mem);
                ReleaseDC(ptr::null_mut(), hdc_screen);
                return Err(AppError::internal(
                    "Gagal mengalokasikan bitmap memory untuk tangkapan layar.",
                ));
            }

            let old_bmp = SelectObject(hdc_mem, hbitmap);

            let blt_res = BitBlt(
                hdc_mem,
                0,
                0,
                w as i32,
                h as i32,
                hdc_screen,
                x,
                y,
                SRCCOPY | CAPTUREBLT,
            );

            if blt_res == 0 {
                SelectObject(hdc_mem, old_bmp);
                DeleteObject(hbitmap);
                DeleteDC(hdc_mem);
                ReleaseDC(ptr::null_mut(), hdc_screen);
                return Err(AppError::internal(
                    "Gagal menyalin piksel layar (BitBlt) dari Windows DC.",
                ));
            }

            let pixels = extract_rgba_from_hbitmap(hdc_mem, hbitmap, w, h)?;

            SelectObject(hdc_mem, old_bmp);
            DeleteObject(hbitmap);
            DeleteDC(hdc_mem);
            ReleaseDC(ptr::null_mut(), hdc_screen);

            Ok(RawCapturedImage {
                width: w,
                height: h,
                rgba_pixels: pixels,
                target_label: Some(label),
            })
        }
    }

    fn capture_window(&self, window_id: u64) -> Result<RawCapturedImage, AppError> {
        use ffi::*;
        use std::ptr;

        unsafe {
            let hwnd = window_id as usize as HWND;
            if IsWindow(hwnd) == 0 {
                return Err(AppError::not_found(format!(
                    "Jendela dengan ID {window_id} tidak valid atau telah ditutup."
                )));
            }

            let mut rect = RECT::default();
            if GetWindowRect(hwnd, &mut rect) == 0 {
                return Err(AppError::internal(
                    "Gagal membaca koordinat batas jendela aplikasi.",
                ));
            }

            let w = (rect.right - rect.left).max(1) as u32;
            let h = (rect.bottom - rect.top).max(1) as u32;

            let title_len = GetWindowTextLengthW(hwnd);
            let title = if title_len > 0 {
                let mut buf = vec![0u16; (title_len + 1) as usize];
                let read = GetWindowTextW(hwnd, buf.as_mut_ptr(), title_len + 1);
                String::from_utf16_lossy(&buf[..read as usize])
                    .trim()
                    .to_string()
            } else {
                format!("Jendela #{window_id}")
            };

            let hdc_screen = GetDC(ptr::null_mut());
            if hdc_screen.is_null() {
                return Err(AppError::internal("Gagal mendapatkan DC layar utama."));
            }

            let hdc_mem = CreateCompatibleDC(hdc_screen);
            if hdc_mem.is_null() {
                ReleaseDC(ptr::null_mut(), hdc_screen);
                return Err(AppError::internal(
                    "Gagal membuat Compatible DC untuk jendela.",
                ));
            }

            let hbitmap = CreateCompatibleBitmap(hdc_screen, w as i32, h as i32);
            if hbitmap.is_null() {
                DeleteDC(hdc_mem);
                ReleaseDC(ptr::null_mut(), hdc_screen);
                return Err(AppError::internal(
                    "Gagal mengalokasikan bitmap memory untuk jendela.",
                ));
            }

            let old_bmp = SelectObject(hdc_mem, hbitmap);

            // First attempt: PrintWindow with PW_RENDERFULLCONTENT (captures DWM-rendered content)
            let print_res = PrintWindow(hwnd, hdc_mem, PW_RENDERFULLCONTENT);

            // Fallback: If PrintWindow fails, BitBlt directly from screen DC at window coordinates
            if print_res == 0 {
                let blt_res = BitBlt(
                    hdc_mem,
                    0,
                    0,
                    w as i32,
                    h as i32,
                    hdc_screen,
                    rect.left,
                    rect.top,
                    SRCCOPY | CAPTUREBLT,
                );
                if blt_res == 0 {
                    SelectObject(hdc_mem, old_bmp);
                    DeleteObject(hbitmap);
                    DeleteDC(hdc_mem);
                    ReleaseDC(ptr::null_mut(), hdc_screen);
                    return Err(AppError::internal(
                        "Gagal mengambil tangkapan layar jendela (PrintWindow & BitBlt keduanya gagal).",
                    ));
                }
            }

            let pixels = extract_rgba_from_hbitmap(hdc_mem, hbitmap, w, h)?;

            SelectObject(hdc_mem, old_bmp);
            DeleteObject(hbitmap);
            DeleteDC(hdc_mem);
            ReleaseDC(ptr::null_mut(), hdc_screen);

            Ok(RawCapturedImage {
                width: w,
                height: h,
                rgba_pixels: pixels,
                target_label: Some(format!("Jendela: {title}")),
            })
        }
    }
}

#[cfg(target_os = "windows")]
#[allow(clippy::chunks_exact_to_as_chunks)]
unsafe fn extract_rgba_from_hbitmap(
    hdc: ffi::HDC,
    hbitmap: ffi::HBITMAP,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, AppError> {
    use ffi::*;

    let mut bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as DWORD,
            biWidth: width as i32,
            biHeight: -(height as i32), // Negative height indicates top-down bitmap
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB,
            biSizeImage: width * height * 4,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [RGBQUAD::default()],
    };

    let total_bytes = (width as usize) * (height as usize) * 4;
    let mut bgra_buffer: Vec<u8> = vec![0u8; total_bytes];

    let lines = GetDIBits(
        hdc,
        hbitmap,
        0,
        height as UINT,
        bgra_buffer.as_mut_ptr() as *mut std::ffi::c_void,
        &mut bmi,
        DIB_RGB_COLORS,
    );

    if lines <= 0 {
        return Err(AppError::internal(
            "Gagal membaca bit piksel dari DIB bitmap (GetDIBits).",
        ));
    }

    // Convert BGRA to RGBA in-place or into a new buffer
    let mut rgba_buffer = vec![0u8; total_bytes];
    for (src, dst) in bgra_buffer
        .chunks_exact(4)
        .zip(rgba_buffer.chunks_exact_mut(4))
    {
        dst[0] = src[2]; // R
        dst[1] = src[1]; // G
        dst[2] = src[0]; // B
        dst[3] = 255; // A (opaque)
    }

    Ok(rgba_buffer)
}

#[cfg(not(target_os = "windows"))]
impl ScreenCaptureEngine for WindowsScreenCaptureEngine {
    fn list_windows(&self) -> Result<Vec<WindowInfo>, AppError> {
        Err(AppError::internal(
            "Windows capture engine hanya didukung pada platform Windows.",
        ))
    }

    fn capture_region(&self, _bounds: Option<CaptureRect>) -> Result<RawCapturedImage, AppError> {
        Err(AppError::internal(
            "Windows capture engine hanya didukung pada platform Windows.",
        ))
    }

    fn capture_window(&self, _window_id: u64) -> Result<RawCapturedImage, AppError> {
        Err(AppError::internal(
            "Windows capture engine hanya didukung pada platform Windows.",
        ))
    }
}
