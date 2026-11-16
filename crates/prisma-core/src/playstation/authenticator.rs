use std::sync::Arc;

use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::domain::Timestamp;
use crate::support::secrets::SecretStore;
use crate::support::{HttpError, decoded};

const AUTH_BASE: &str = "https://ca.account.sony.com/api/authz/v3/oauth";
const CLIENT_ID: &str = "09515159-7237-4370-9b40-3806e67c0891";
const CLIENT_CREDENTIALS: &str = "MDk1MTUxNTktNzIzNy00MzcwLTliNDAtMzgwNmU2N2MwODkxOnVjUGprYTV0bnRCMktxc1A=";
const REDIRECT_URI: &str = "com.scee.psxandroid.scecompcall://redirect";
const SCOPE: &str = "psn:mobile.v2.core psn:clientapp";
const SECRET_ACCOUNT: &str = "psn";
const SERVICE: &str = "PlayStation Network";

/// Marge qui évite d'utiliser un jeton sur le point d'expirer.
fn expiry_margin() -> Duration {
    Duration::seconds(60)
}

#[derive(Debug, thiserror::Error)]
pub enum PsnError {
    #[error("Jeton NPSSO invalide ou expiré. Reconnectez-vous sur playstation.com puis récupérez-en un nouveau.")]
    InvalidNpsso,
    #[error("Compte PlayStation non connecté.")]
    NotConnected,
    #[error(transparent)]
    Http(#[from] HttpError),
}

#[derive(Clone, Serialize, Deserialize)]
struct Tokens {
    access: String,
    access_expiry: Timestamp,
    refresh: String,
    refresh_expiry: Timestamp,
}

/// Les jetons sont lus dans le coffre à la première utilisation : l'accès au trousseau
/// peut attendre une autorisation de l'utilisateur.
enum Session {
    NotLoaded,
    Loaded(Option<Tokens>),
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: i64,
    refresh_token: String,
    refresh_token_expires_in: i64,
}

/// Connexion au PlayStation Network par le même flux OAuth que l'application mobile PlayStation.
pub struct PsnAuthenticator {
    secrets: Arc<dyn SecretStore>,
    client: reqwest::Client,
    session: Mutex<Session>,
}

impl PsnAuthenticator {
    pub fn new(secrets: Arc<dyn SecretStore>) -> Self {
        PsnAuthenticator { secrets, client: reqwest::Client::new(), session: Mutex::new(Session::NotLoaded) }
    }

    pub async fn is_connected(&self) -> bool {
        self.current_tokens().await.is_some_and(|tokens| tokens.refresh_expiry > Utc::now())
    }

    /// Échange le jeton NPSSO, copié depuis le site de Sony, contre des jetons d'accès durables.
    pub async fn connect(&self, npsso: &str) -> Result<(), PsnError> {
        let code = self.authorization_code(npsso.trim()).await?;
        self.request_tokens(&[
            ("code", &code),
            ("redirect_uri", REDIRECT_URI),
            ("grant_type", "authorization_code"),
            ("token_format", "jwt"),
        ])
        .await?;
        Ok(())
    }

    pub async fn disconnect(&self) {
        *self.session.lock().await = Session::Loaded(None);
        self.secrets.delete(SECRET_ACCOUNT);
    }

    pub(crate) async fn access_token(&self) -> Result<String, PsnError> {
        let now = Utc::now();
        let current = self.current_tokens().await.filter(|tokens| tokens.refresh_expiry > now).ok_or(PsnError::NotConnected)?;
        if current.access_expiry > now {
            return Ok(current.access);
        }
        let refreshed = self
            .request_tokens(&[
                ("refresh_token", &current.refresh),
                ("grant_type", "refresh_token"),
                ("scope", SCOPE),
                ("token_format", "jwt"),
            ])
            .await?;
        Ok(refreshed.access)
    }

    async fn current_tokens(&self) -> Option<Tokens> {
        let mut session = self.session.lock().await;
        if let Session::NotLoaded = *session {
            let stored = self.secrets.load(SECRET_ACCOUNT).and_then(|json| serde_json::from_str(&json).ok());
            *session = Session::Loaded(stored);
        }
        match &*session {
            Session::Loaded(tokens) => tokens.clone(),
            Session::NotLoaded => None,
        }
    }

    /// Sony répond par une redirection dont l'adresse porte le code d'autorisation.
    async fn authorization_code(&self, npsso: &str) -> Result<String, PsnError> {
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|source| PsnError::Http(HttpError::Transport { service: SERVICE, source }))?;
        let response = client
            .get(format!("{AUTH_BASE}/authorize"))
            .query(&[
                ("access_type", "offline"),
                ("client_id", CLIENT_ID),
                ("redirect_uri", REDIRECT_URI),
                ("response_type", "code"),
                ("scope", SCOPE),
            ])
            .header(reqwest::header::COOKIE, format!("npsso={npsso}"))
            .send()
            .await
            .map_err(|source| HttpError::Transport { service: SERVICE, source })?;

        let location = response.headers().get(reqwest::header::LOCATION).and_then(|value| value.to_str().ok());
        let code = location
            .and_then(|location| url::Url::parse(location).ok())
            .and_then(|url| url.query_pairs().find(|(name, _)| name == "code").map(|(_, code)| code.into_owned()));
        code.ok_or(PsnError::InvalidNpsso)
    }

    async fn request_tokens(&self, form: &[(&str, &str)]) -> Result<Tokens, PsnError> {
        let request = self
            .client
            .post(format!("{AUTH_BASE}/token"))
            .header(reqwest::header::AUTHORIZATION, format!("Basic {CLIENT_CREDENTIALS}"))
            .form(form);
        let response: TokenResponse = decoded(request, SERVICE).await.map_err(|error| match error.status() {
            Some(400 | 401) => PsnError::InvalidNpsso,
            _ => PsnError::Http(error),
        })?;

        let now = Utc::now();
        let tokens = Tokens {
            access: response.access_token,
            access_expiry: now + Duration::seconds(response.expires_in) - expiry_margin(),
            refresh: response.refresh_token,
            refresh_expiry: now + Duration::seconds(response.refresh_token_expires_in) - expiry_margin(),
        };
        if let Ok(json) = serde_json::to_string(&tokens) {
            self.secrets.save(SECRET_ACCOUNT, &json);
        }
        *self.session.lock().await = Session::Loaded(Some(tokens.clone()));
        Ok(tokens)
    }
}
