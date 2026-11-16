use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::Deserialize;

use super::{PsnAuthenticator, PsnError};
use crate::domain::{Game, GameId, Platform, Trophy, TrophyTier, system};
use crate::support::decoded;

const API_BASE: &str = "https://m.np.playstation.com/api/trophy/v1";
const PAGE_SIZE: usize = 800;
const SERVICE: &str = "PlayStation Network";

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TitleSummary {
    np_service_name: String,
    pub np_communication_id: String,
    trophy_title_name: String,
    trophy_title_icon_url: Option<String>,
    trophy_title_platform: String,
    pub last_updated_date_time: Option<String>,
}

#[derive(Clone)]
pub struct PsnTrophyClient {
    authenticator: Arc<PsnAuthenticator>,
    client: reqwest::Client,
}

impl PsnTrophyClient {
    pub fn new(authenticator: Arc<PsnAuthenticator>) -> Self {
        PsnTrophyClient { authenticator, client: reqwest::Client::new() }
    }

    pub async fn titles(&self) -> Result<Vec<TitleSummary>, PsnError> {
        let mut titles = Vec::new();
        loop {
            let offset = titles.len().to_string();
            let page: TitlesPage = self.get("/users/me/trophyTitles", &[("limit", &PAGE_SIZE.to_string()), ("offset", &offset)]).await?;
            let page_is_empty = page.trophy_titles.is_empty();
            titles.extend(page.trophy_titles);
            if page_is_empty || titles.len() >= page.total_item_count.unwrap_or(0) {
                return Ok(titles);
            }
        }
    }

    pub async fn game(&self, title: &TitleSummary) -> Result<Game, PsnError> {
        let id = &title.np_communication_id;
        let query = [("npServiceName", title.np_service_name.as_str())];
        let definitions_path = format!("/npCommunicationIds/{id}/trophyGroups/all/trophies");
        let progress_path = format!("/users/me/npCommunicationIds/{id}/trophyGroups/all/trophies");
        let (definitions, progress) = tokio::try_join!(
            self.get::<DefinitionsResponse>(&definitions_path, &query),
            self.get::<ProgressResponse>(&progress_path, &query),
        )?;

        let progress_by_id: HashMap<u32, ProgressState> = progress.trophies.into_iter().map(|state| (state.trophy_id, state)).collect();
        let trophies = definitions
            .trophies
            .into_iter()
            .map(|definition| {
                let progress = progress_by_id.get(&definition.trophy_id);
                definition.into_trophy(progress)
            })
            .collect();
        let systems = title.trophy_title_platform.split(',').map(system_name).collect();
        Ok(Game {
            cover_url: title.trophy_title_icon_url.clone(),
            ..Game::new(GameId::playstation(id), Platform::Playstation, &title.trophy_title_name, systems, trophies)
        })
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str, query: &[(&str, &str)]) -> Result<T, PsnError> {
        let token = self.authenticator.access_token().await?;
        let request =
            self.client.get(format!("{API_BASE}{path}")).query(query).bearer_auth(token).header(reqwest::header::ACCEPT_LANGUAGE, "fr-FR");
        Ok(decoded(request, SERVICE).await?)
    }
}

fn system_name(platform_code: &str) -> String {
    if platform_code == "PSVITA" { system::PS_VITA.into() } else { platform_code.into() }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TitlesPage {
    trophy_titles: Vec<TitleSummary>,
    total_item_count: Option<usize>,
}

#[derive(Deserialize)]
struct DefinitionsResponse {
    trophies: Vec<Definition>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Definition {
    trophy_id: u32,
    trophy_hidden: Option<bool>,
    trophy_type: String,
    trophy_name: Option<String>,
    trophy_detail: Option<String>,
    trophy_icon_url: Option<String>,
}

impl Definition {
    fn into_trophy(self, progress: Option<&ProgressState>) -> Trophy {
        let is_earned = progress.and_then(|state| state.earned).unwrap_or(false);
        let earned_at = progress
            .and_then(|state| state.earned_date_time.as_deref())
            .and_then(|date| DateTime::parse_from_rfc3339(date).ok())
            .map(|date| date.with_timezone(&Utc));
        Trophy {
            is_hidden: self.trophy_hidden.unwrap_or(false),
            icon_url: self.trophy_icon_url.clone(),
            locked_icon_url: self.trophy_icon_url,
            is_earned,
            earned_at: earned_at.filter(|_| is_earned),
            ..Trophy::new(
                self.trophy_id.to_string(),
                self.trophy_name.unwrap_or_default(),
                self.trophy_detail.unwrap_or_default(),
                TrophyTier::from_name(&self.trophy_type).unwrap_or_default(),
            )
        }
    }
}

#[derive(Deserialize)]
struct ProgressResponse {
    trophies: Vec<ProgressState>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProgressState {
    trophy_id: u32,
    earned: Option<bool>,
    earned_date_time: Option<String>,
}
