use super::store::CredentialStore;
use crate::domain::error::AppError;

/// Implementasi produksi native Windows Credential Manager menggunakan API Advapi32.
#[derive(Debug, Clone, Default)]
pub struct WindowsCredentialStore;

impl WindowsCredentialStore {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(target_os = "windows")]
mod ffi {
    use std::ffi::c_void;

    #[repr(C)]
    pub struct FILETIME {
        pub dwLowDateTime: u32,
        pub dwHighDateTime: u32,
    }

    #[repr(C)]
    pub struct CREDENTIALW {
        pub Flags: u32,
        pub Type: u32,
        pub TargetName: *mut u16,
        pub Comment: *mut u16,
        pub LastWritten: FILETIME,
        pub CredentialBlobSize: u32,
        pub CredentialBlob: *mut u8,
        pub Persist: u32,
        pub AttributeCount: u32,
        pub Attributes: *mut c_void,
        pub TargetAlias: *mut u16,
        pub UserName: *mut u16,
    }

    pub const CRED_TYPE_GENERIC: u32 = 1;
    pub const CRED_PERSIST_LOCAL_MACHINE: u32 = 2;
    pub const ERROR_NOT_FOUND: u32 = 1168;
    pub const ERROR_ACCESS_DENIED: u32 = 5;

    #[link(name = "advapi32")]
    extern "system" {
        pub fn CredWriteW(credential: *const CREDENTIALW, flags: u32) -> i32;
        pub fn CredReadW(
            target_name: *const u16,
            cred_type: u32,
            flags: u32,
            credential: *mut *mut CREDENTIALW,
        ) -> i32;
        pub fn CredDeleteW(target_name: *const u16, cred_type: u32, flags: u32) -> i32;
        pub fn CredFree(buffer: *mut c_void);
    }

    extern "system" {
        pub fn GetLastError() -> u32;
    }
}

#[cfg(target_os = "windows")]
impl CredentialStore for WindowsCredentialStore {
    fn set_secret(&self, target_name: &str, secret: &str) -> Result<(), AppError> {
        use ffi::*;
        let mut target_wide: Vec<u16> = target_name.encode_utf16().chain(Some(0)).collect();
        let mut username_wide: Vec<u16> = "survey_copilot".encode_utf16().chain(Some(0)).collect();
        let secret_bytes = secret.as_bytes();

        let cred = CREDENTIALW {
            Flags: 0,
            Type: CRED_TYPE_GENERIC,
            TargetName: target_wide.as_mut_ptr(),
            Comment: std::ptr::null_mut(),
            LastWritten: FILETIME {
                dwLowDateTime: 0,
                dwHighDateTime: 0,
            },
            CredentialBlobSize: secret_bytes.len() as u32,
            CredentialBlob: secret_bytes.as_ptr() as *mut u8,
            Persist: CRED_PERSIST_LOCAL_MACHINE,
            AttributeCount: 0,
            Attributes: std::ptr::null_mut(),
            TargetAlias: std::ptr::null_mut(),
            UserName: username_wide.as_mut_ptr(),
        };

        let res = unsafe { CredWriteW(&cred, 0) };
        if res == 0 {
            let err = unsafe { GetLastError() };
            return Err(match err {
                ERROR_ACCESS_DENIED => AppError::credential_access_denied(
                    "Akses ditolak oleh Windows Credential Manager saat menyimpan kredensial.",
                ),
                _ => AppError::credential_error(format!(
                    "Gagal menyimpan kredensial ke Windows Credential Manager (kode error OS: {err})."
                )),
            });
        }

        Ok(())
    }

    fn get_secret(&self, target_name: &str) -> Result<String, AppError> {
        use ffi::*;
        let target_wide: Vec<u16> = target_name.encode_utf16().chain(Some(0)).collect();
        let mut cred_ptr: *mut CREDENTIALW = std::ptr::null_mut();

        let res = unsafe { CredReadW(target_wide.as_ptr(), CRED_TYPE_GENERIC, 0, &mut cred_ptr) };

        if res == 0 {
            let err = unsafe { GetLastError() };
            return Err(match err {
                ERROR_NOT_FOUND => AppError::credential_not_found(
                    "Kredensial tidak ditemukan di Windows Credential Manager.",
                ),
                ERROR_ACCESS_DENIED => AppError::credential_access_denied(
                    "Akses ditolak saat membaca kredensial dari Windows Credential Manager.",
                ),
                _ => AppError::credential_error(format!(
                    "Gagal membaca kredensial dari Windows Credential Manager (kode error OS: {err})."
                )),
            });
        }

        if cred_ptr.is_null() {
            return Err(AppError::credential_error(
                "Pointer kredensial null dari Windows Credential Manager.",
            ));
        }

        let slice = unsafe {
            std::slice::from_raw_parts(
                (*cred_ptr).CredentialBlob,
                (*cred_ptr).CredentialBlobSize as usize,
            )
        };

        let secret = std::str::from_utf8(slice)
            .map_err(|_| {
                AppError::credential_error("Data kredensial bukan format UTF-8 yang valid.")
            })?
            .to_string();

        unsafe {
            CredFree(cred_ptr as *mut _);
        }

        Ok(secret)
    }

    fn delete_secret(&self, target_name: &str) -> Result<(), AppError> {
        use ffi::*;
        let target_wide: Vec<u16> = target_name.encode_utf16().chain(Some(0)).collect();

        let res = unsafe { CredDeleteW(target_wide.as_ptr(), CRED_TYPE_GENERIC, 0) };
        if res == 0 {
            let err = unsafe { GetLastError() };
            if err == ERROR_NOT_FOUND {
                // Not found is treated as already deleted / absent
                return Ok(());
            }
            return Err(match err {
                ERROR_ACCESS_DENIED => AppError::credential_access_denied(
                    "Akses ditolak saat menghapus kredensial dari Windows Credential Manager.",
                ),
                _ => AppError::credential_error(format!(
                    "Gagal menghapus kredensial dari Windows Credential Manager (kode error OS: {err})."
                )),
            });
        }

        Ok(())
    }

    fn exists(&self, target_name: &str) -> Result<bool, AppError> {
        match self.get_secret(target_name) {
            Ok(_) => Ok(true),
            Err(err) if err.code == "CREDENTIAL_NOT_FOUND" => Ok(false),
            Err(err) => Err(err),
        }
    }
}

#[cfg(not(target_os = "windows"))]
impl CredentialStore for WindowsCredentialStore {
    fn set_secret(&self, _target_name: &str, _secret: &str) -> Result<(), AppError> {
        Err(AppError::credential_error(
            "Windows Credential Manager hanya didukung di sistem operasi Windows.",
        ))
    }

    fn get_secret(&self, _target_name: &str) -> Result<String, AppError> {
        Err(AppError::credential_error(
            "Windows Credential Manager hanya didukung di sistem operasi Windows.",
        ))
    }

    fn delete_secret(&self, _target_name: &str) -> Result<(), AppError> {
        Err(AppError::credential_error(
            "Windows Credential Manager hanya didukung di sistem operasi Windows.",
        ))
    }

    fn exists(&self, _target_name: &str) -> Result<bool, AppError> {
        Err(AppError::credential_error(
            "Windows Credential Manager hanya didukung di sistem operasi Windows.",
        ))
    }
}
