use std::collections::HashMap;
use std::sync::Mutex;

/// Coffre où sont rangés les secrets, comme les jetons PlayStation.
pub trait SecretStore: Send + Sync {
    fn load(&self, account: &str) -> Option<String>;
    fn save(&self, account: &str, secret: &str);
    fn delete(&self, account: &str);
}

/// Coffre éphémère, pour une bibliothèque isolée qui ne doit pas toucher au vrai trousseau.
#[derive(Default)]
pub struct EphemeralSecretStore {
    secrets: Mutex<HashMap<String, String>>,
}

impl SecretStore for EphemeralSecretStore {
    fn load(&self, account: &str) -> Option<String> {
        self.secrets.lock().ok()?.get(account).cloned()
    }

    fn save(&self, account: &str, secret: &str) {
        if let Ok(mut secrets) = self.secrets.lock() {
            secrets.insert(account.to_owned(), secret.to_owned());
        }
    }

    fn delete(&self, account: &str) {
        if let Ok(mut secrets) = self.secrets.lock() {
            secrets.remove(account);
        }
    }
}
