use crate::config::ConfigError;
use crate::content_storage::ContentStorageError;
use crate::did_registry::DidRegistryError;

pub fn map_content_store_validation_error(error: ContentStorageError) -> ConfigError {
    match error {
        ContentStorageError::InvalidPayload(detail) => corrupt_payload(
            "content-storage",
            "content_storage_corrupt_payload_rejected",
            detail,
        ),
        ContentStorageError::Io(detail) => io_error(
            "content-storage",
            "content_storage_io_error",
            detail,
        ),
        other => compatibility_error("content-storage", "content_storage_compatibility_failed", other),
    }
}

pub fn map_did_registry_store_validation_error(error: DidRegistryError) -> ConfigError {
    match error {
        DidRegistryError::PersistenceInvalidPayload(detail) => corrupt_payload(
            "did-registry",
            "did_registry_corrupt_payload_rejected",
            detail,
        ),
        DidRegistryError::PersistenceIo(detail) => io_error(
            "did-registry",
            "did_registry_io_error",
            detail,
        ),
        other => compatibility_error("did-registry", "did_registry_compatibility_failed", other),
    }
}

fn compatibility_error(store: &'static str, reason_code: &'static str, other: impl ToString) -> ConfigError {
    ConfigError::RuntimeStoreCompatibility {
        store,
        reason_code,
        detail: other.to_string(),
    }
}

fn corrupt_payload(store: &'static str, reason_code: &'static str, detail: String) -> ConfigError {
    ConfigError::RuntimeStoreCorruptPayload { store, reason_code, detail }
}

fn io_error(store: &'static str, reason_code: &'static str, detail: String) -> ConfigError {
    ConfigError::RuntimeStoreCompatibility { store, reason_code, detail }
}
