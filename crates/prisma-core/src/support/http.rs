use serde::de::DeserializeOwned;

#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    #[error("{service} : erreur HTTP {status}.")]
    Status { service: &'static str, status: u16 },
    #[error("{service} : {source}")]
    Transport {
        service: &'static str,
        #[source]
        source: reqwest::Error,
    },
}

impl HttpError {
    pub fn status(&self) -> Option<u16> {
        match self {
            HttpError::Status { status, .. } => Some(*status),
            HttpError::Transport { .. } => None,
        }
    }
}

/// Envoie la requête et décode la réponse JSON, en traitant tout statut autre que 200 comme une erreur.
pub(crate) async fn decoded<T: DeserializeOwned>(request: reqwest::RequestBuilder, service: &'static str) -> Result<T, HttpError> {
    let transport = |source| HttpError::Transport { service, source };
    let response = request.send().await.map_err(transport)?;
    let status = response.status();
    if status != reqwest::StatusCode::OK {
        return Err(HttpError::Status { service, status: status.as_u16() });
    }
    response.json().await.map_err(transport)
}
