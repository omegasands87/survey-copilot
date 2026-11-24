use super::store::CredentialStore;
use crate::domain::error::AppError;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Implementasi in-memory untuk pengujian terisolasi dan pengujian lingkungan non-Windows.
///
/// PERINGATAN ARSITEKTUR:
/// Komponen ini HANYA digunakan dalam unit/integration test harness dan lingkungan non-Windows.
/// Komponen ini TIDAK PERNAH digunakan sebagai penyimpanan produksi pada rilis Windows 11.
#[derive(Debug, Clone, Default)]
pub struct InMemoryCredentialStore {
    storage: Arc<RwLock<HashMap<String, String>>>,
    fail_delete: Arc<RwLock<bool>>,
}

impl InMemoryCredentialStore {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
            fail_delete: Arc::new(RwLock::new(false)),
        }
    }

    /// Deterministic fault injection for deletion lifecycle tests only.
    pub fn set_delete_failure(&self, fail: bool) {
        if let Ok(mut state) = self.fail_delete.write() {
            *state = fail;
        }
    }
}

impl CredentialStore for InMemoryCredentialStore {
    fn set_secret(&self, target_name: &str, secret: &str) -> Result<(), AppError> {
        let mut map = self.storage.write().map_err(|_| {
            AppError::internal("Gagal memperoleh lock penyimpanan kredensial in-memory.")
        })?;
        map.insert(target_name.to_string(), secret.to_string());
        Ok(())
    }

    fn get_secret(&self, target_name: &str) -> Result<String, AppError> {
        let map = self.storage.read().map_err(|_| {
            AppError::internal("Gagal memperoleh lock penyimpanan kredensial in-memory.")
        })?;
        map.get(target_name).cloned().ok_or_else(|| {
            AppError::credential_not_found("Kredensial tidak ditemukan di penyimpanan in-memory.")
        })
    }

    fn delete_secret(&self, target_name: &str) -> Result<(), AppError> {
        let should_fail = *self.fail_delete.read().map_err(|_| {
            AppError::internal("Gagal memperoleh status fault injection credential.")
        })?;
        if should_fail {
            return Err(AppError::credential_error(
                "Simulated credential-store deletion failure.",
            ));
        }

        let mut map = self.storage.write().map_err(|_| {
            AppError::internal("Gagal memperoleh lock penyimpanan kredensial in-memory.")
        })?;
        map.remove(target_name);
        Ok(())
    }

    fn exists(&self, target_name: &str) -> Result<bool, AppError> {
        let map = self.storage.read().map_err(|_| {
            AppError::internal("Gagal memperoleh lock penyimpanan kredensial in-memory.")
        })?;
        Ok(map.contains_key(target_name))
    }
}
