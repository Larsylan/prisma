use std::collections::HashMap;

use serde::{Deserialize, Deserializer};

use super::SteamAchievement;
use crate::support::{HttpError, decoded};

const API_BASE: &str = "https://api.steampowered.com";
const SERVICE: &str = "Steam Web API";

#[derive(Clone, Debug)]
pub struct Credentials {
    key: String,
    steam_id64: String,
}

impl Credentials {
    pub fn new(key: &str, steam_id64: &str) -> Option<Self> {
        let key = key.trim();
        (!key.is_empty() && !steam_id64.is_empty()).then(|| Credentials { key: key.to_owned(), steam_id64: steam_id64.to_owned() })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct OwnedGame {
    #[serde(rename = "appid")]
    pub app_id: u32,
    pub name: Option<String>,
    #[serde(rename = "playtime_forever")]
    playtime_minutes: Option<u64>,
    has_community_visible_stats: Option<bool>,
}

impl OwnedGame {
    /// Seuls les jeux lancés au moins une fois et dont les statistiques sont publiques ont des succès lisibles.
    pub fn may_have_achievements(&self) -> bool {
        self.playtime_minutes.unwrap_or(0) > 0 && self.has_community_visible_stats.unwrap_or(true)
    }
}

#[derive(Clone, Debug)]
pub struct GameAchievements {
    pub schema_name: String,
    pub achievements: Vec<SteamAchievement>,
}

/// Steam Web API : historique complet des succès, y compris des jeux jamais lancés sur cet ordinateur.
#[derive(Clone, Debug, Default)]
pub struct SteamWebApi {
    client: reqwest::Client,
}

impl SteamWebApi {
    pub async fn owned_games(&self, credentials: &Credentials) -> Result<Vec<OwnedGame>, HttpError> {
        let response: OwnedGamesResponse = self
            .get(
                "/IPlayerService/GetOwnedGames/v1/",
                &[
                    ("key", credentials.key.as_str()),
                    ("steamid", &credentials.steam_id64),
                    ("include_appinfo", "1"),
                    ("include_played_free_games", "1"),
                ],
            )
            .await?;
        Ok(response.response.games.unwrap_or_default())
    }

    /// Retourne `None` si le jeu n'a pas de succès.
    pub async fn achievements(&self, app_id: u32, credentials: &Credentials) -> Result<Option<GameAchievements>, HttpError> {
        let app_id = app_id.to_string();
        let schema: SchemaResponse = self
            .get("/ISteamUserStats/GetSchemaForGame/v2/", &[("key", credentials.key.as_str()), ("appid", &app_id), ("l", "french")])
            .await?;
        let Some(game) = schema.game else { return Ok(None) };
        let definitions = game.available_game_stats.and_then(|stats| stats.achievements).unwrap_or_default();
        if definitions.is_empty() {
            return Ok(None);
        }

        let player: PlayerAchievementsResponse = self
            .get(
                "/ISteamUserStats/GetPlayerAchievements/v1/",
                &[("key", credentials.key.as_str()), ("steamid", &credentials.steam_id64), ("appid", &app_id)],
            )
            .await?;
        let progress: HashMap<String, PlayerAchievement> =
            player.playerstats.achievements.unwrap_or_default().into_iter().map(|state| (state.apiname.clone(), state)).collect();

        let achievements = definitions
            .into_iter()
            .map(|definition| {
                let state = progress.get(&definition.name);
                SteamAchievement {
                    name: definition.display_name.unwrap_or_else(|| definition.name.clone()),
                    detail: definition.description.unwrap_or_default(),
                    is_hidden: definition.hidden.unwrap_or(0) != 0,
                    icon_url: definition.icon,
                    locked_icon_url: definition.icongray,
                    api_name: definition.name,
                    ..SteamAchievement::default()
                }
                .unlocked_at(
                    state.is_some_and(|state| state.achieved == 1),
                    SteamAchievement::unlock_date(state.and_then(|state| state.unlocktime)),
                )
            })
            .collect();
        Ok(Some(GameAchievements { schema_name: game.game_name.unwrap_or_default(), achievements }))
    }

    /// Pourcentage de joueurs ayant obtenu chaque succès, sans clé d'API.
    pub async fn unlock_percentages(&self, app_id: u32) -> Result<HashMap<String, f64>, HttpError> {
        let response: GlobalPercentagesResponse =
            self.get("/ISteamUserStats/GetGlobalAchievementPercentagesForApp/v2/", &[("gameid", &app_id.to_string())]).await?;
        Ok(response.achievementpercentages.achievements.into_iter().map(|entry| (entry.name, entry.percent)).collect())
    }

    /// Nom public d'un jeu dans la boutique Steam.
    pub async fn store_title(&self, app_id: u32) -> Option<String> {
        let app_id = app_id.to_string();
        let request = self.client.get("https://store.steampowered.com/api/appdetails").query(&[
            ("appids", app_id.as_str()),
            ("filters", "basic"),
            ("l", "french"),
        ]);
        let mut response: HashMap<String, StoreEntry> = decoded(request, SERVICE).await.ok()?;
        response.remove(&app_id)?.data?.name
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str, query: &[(&str, &str)]) -> Result<T, HttpError> {
        decoded(self.client.get(format!("{API_BASE}{path}")).query(query), SERVICE).await
    }
}

#[derive(Deserialize)]
struct OwnedGamesResponse {
    response: OwnedGamesContent,
}

#[derive(Deserialize)]
struct OwnedGamesContent {
    games: Option<Vec<OwnedGame>>,
}

#[derive(Deserialize)]
struct SchemaResponse {
    game: Option<SchemaGame>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SchemaGame {
    game_name: Option<String>,
    available_game_stats: Option<SchemaStats>,
}

#[derive(Deserialize)]
struct SchemaStats {
    achievements: Option<Vec<AchievementDefinition>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AchievementDefinition {
    name: String,
    display_name: Option<String>,
    description: Option<String>,
    hidden: Option<u8>,
    icon: Option<String>,
    icongray: Option<String>,
}

#[derive(Deserialize)]
struct PlayerAchievementsResponse {
    playerstats: PlayerStats,
}

#[derive(Deserialize)]
struct PlayerStats {
    achievements: Option<Vec<PlayerAchievement>>,
}

#[derive(Deserialize)]
struct PlayerAchievement {
    apiname: String,
    achieved: u8,
    unlocktime: Option<i64>,
}

#[derive(Deserialize)]
struct GlobalPercentagesResponse {
    achievementpercentages: GlobalPercentages,
}

#[derive(Deserialize)]
struct GlobalPercentages {
    achievements: Vec<GlobalPercentage>,
}

#[derive(Deserialize)]
struct GlobalPercentage {
    name: String,
    #[serde(deserialize_with = "lenient_percentage")]
    percent: f64,
}

#[derive(Deserialize)]
struct StoreEntry {
    data: Option<StoreDetails>,
}

#[derive(Deserialize)]
struct StoreDetails {
    name: Option<String>,
}

/// Steam renvoie les pourcentages tantôt en nombre, tantôt en texte. Une valeur illisible
/// est traitée comme un succès obtenu par tous, donc classé bronze.
fn lenient_percentage<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f64, D::Error> {
    const FALLBACK: f64 = 100.0;
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Number {
        Float(f64),
        Text(String),
    }
    Ok(match Number::deserialize(deserializer)? {
        Number::Float(value) => value,
        Number::Text(text) => text.parse().unwrap_or(FALLBACK),
    })
}
