use crate::{SoftwareVault, VaultError};
use ockam_vault_core::ErrorVault;

impl SoftwareVault {
    /// Return the error domain for this Vault
    pub fn error_domain_static() -> &'static str {
        VaultError::DOMAIN_NAME
    }
}

impl ErrorVault for SoftwareVault {
    fn error_domain(&self) -> &'static str {
        Self::error_domain_static()
    }
}
