pub mod codec;
pub mod engine;
pub mod in_memory;
pub mod windows;

pub use codec::{bytes_to_base64, encode_rgba_to_png, png_to_data_url};
pub use engine::{RawCapturedImage, ScreenCaptureEngine};
pub use in_memory::InMemoryScreenCaptureEngine;
pub use windows::WindowsScreenCaptureEngine;
