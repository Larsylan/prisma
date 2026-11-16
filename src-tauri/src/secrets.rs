use prisma_core::support::secrets::SecretStore;

/// Coffre du système : Trousseau sur macOS, Gestionnaire d'identification sur Windows, Secret Service sur Linux.
pub struct KeyringSecretStore {
    service: String,
}

impl KeyringSecretStore {
    pub fn new(service: &str) -> Self {
        KeyringSecretStore { service: service.to_owned() }
    }

    fn entry(&self, account: &str) -> Option<keyring::Entry> {
        keyring::Entry::new(&self.service, account).inspect_err(|error| log::error!("Coffre inaccessible : {error}")).ok()
    }
}

impl SecretStore for KeyringSecretStore {
    fn load(&self, account: &str) -> Option<String> {
        match self.entry(account)?.get_password() {
            Ok(secret) => Some(secret),
            Err(keyring::Error::NoEntry) => None,
            Err(error) => {
                log::error!("Lecture du coffre impossible : {error}");
                None
            }
        }
    }

    fn save(&self, account: &str, secret: &str) {
        if let Some(Err(error)) = self.entry(account).map(|entry| entry.set_password(secret)) {
            log::error!("Écriture dans le coffre impossible : {error}");
        }
    }

    fn delete(&self, account: &str) {
        match self.entry(account).map(|entry| entry.delete_credential()) {
            Some(Err(keyring::Error::NoEntry)) | Some(Ok(())) | None => {}
            Some(Err(error)) => log::error!("Suppression dans le coffre impossible : {error}"),
        }
    }
}
