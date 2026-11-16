use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use prisma_core::steam::{Credentials, GameAchievements, OwnedGame, SteamClientCache, SteamWebApi};
use prisma_core::support::secrets::SecretStore;
use tokio::task::JoinSet;

use super::status::StatusBoard;
use super::steam_recorder::SteamRecorder;
use crate::settings::SettingsStore;

const CONCURRENT_REQUESTS: usize = 6;
/// La clé Steam Web API est rangée dans le coffre du système, comme le jeton PlayStation.
const API_KEY_ACCOUNT: &str = "steam-api-key";

/// Importe l'historique complet des succès Steam lorsqu'une clé Steam Web API est renseignée.
pub struct SteamWebImporter {
    api: SteamWebApi,
    steam_cache: Option<SteamClientCache>,
    recorder: Arc<SteamRecorder>,
    settings: Arc<SettingsStore>,
    secrets: Arc<dyn SecretStore>,
    status: Arc<StatusBoard>,
    is_importing: AtomicBool,
}

impl SteamWebImporter {
    pub fn new(
        api: SteamWebApi,
        steam_cache: Option<SteamClientCache>,
        recorder: Arc<SteamRecorder>,
        settings: Arc<SettingsStore>,
        secrets: Arc<dyn SecretStore>,
        status: Arc<StatusBoard>,
    ) -> Self {
        SteamWebImporter { api, steam_cache, recorder, settings, secrets, status, is_importing: AtomicBool::new(false) }
    }

    pub fn api_key(&self) -> String {
        self.secrets.load(API_KEY_ACCOUNT).unwrap_or_default()
    }

    pub fn set_api_key(&self, key: &str) {
        let key = key.trim();
        if key.is_empty() {
            self.secrets.delete(API_KEY_ACCOUNT);
        } else {
            self.secrets.save(API_KEY_ACCOUNT, key);
        }
    }

    pub fn detected_steam_id64(&self) -> Option<String> {
        self.steam_cache.as_ref()?.current_user().map(|user| user.steam_id64.to_string())
    }

    pub async fn import_history(&self) {
        let Some(credentials) = self.credentials() else { return };
        if self.is_importing.swap(true, Ordering::AcqRel) {
            return;
        }
        self.status.update(|status| status.steam_web_importing = true);
        let message = match self.import(&credentials).await {
            Ok(count) => format!("Historique Steam synchronisé ({count} jeux joués)."),
            Err(error) => format!("Steam Web : {error}"),
        };
        self.status.update(|status| {
            status.steam_web = message;
            status.steam_web_importing = false;
        });
        self.is_importing.store(false, Ordering::Release);
    }

    async fn import(&self, credentials: &Credentials) -> Result<usize, prisma_core::support::HttpError> {
        let played: Vec<OwnedGame> =
            self.api.owned_games(credentials).await?.into_iter().filter(OwnedGame::may_have_achievements).collect();
        let mut analyzed = 0;
        for batch in played.chunks(CONCURRENT_REQUESTS) {
            for (owned, achievements) in self.fetch_achievements(batch, credentials).await {
                self.record(&owned, achievements);
            }
            analyzed += batch.len();
            let progress = format!("Steam Web : {analyzed}/{} jeux analysés…", played.len());
            self.status.update(|status| status.steam_web = progress);
        }
        Ok(played.len())
    }

    async fn fetch_achievements(&self, games: &[OwnedGame], credentials: &Credentials) -> Vec<(OwnedGame, Option<GameAchievements>)> {
        let mut requests = JoinSet::new();
        for game in games.iter().cloned() {
            let api = self.api.clone();
            let credentials = credentials.clone();
            requests.spawn(async move {
                let achievements = api.achievements(game.app_id, &credentials).await.ok().flatten();
                (game, achievements)
            });
        }
        requests.join_all().await
    }

    fn record(&self, owned: &OwnedGame, achievements: Option<GameAchievements>) {
        let Some(achievements) = achievements else { return };
        if let Some(name) = &owned.name {
            self.recorder.titles.remember(name, owned.app_id);
        }
        let title = owned.name.clone().unwrap_or_else(|| self.recorder.titles.title(owned.app_id, &achievements.schema_name));
        self.recorder.record(owned.app_id, &title, &achievements.achievements, false);
    }

    fn credentials(&self) -> Option<Credentials> {
        let settings = self.settings.get();
        let steam_id64 = Some(settings.steam_id64).filter(|id| !id.is_empty()).or_else(|| self.detected_steam_id64())?;
        Credentials::new(&self.api_key(), &steam_id64)
    }
}

#[cfg(test)]
mod tests {
    use prisma_core::support::secrets::EphemeralSecretStore;

    use super::*;
    use crate::sync::SteamTitles;
    use crate::testing::TestApp;

    #[test]
    fn api_key_is_kept_in_the_secret_store_and_trimmed() {
        let directory = tempfile::tempdir().unwrap();
        let app = TestApp::new(directory.path());
        let titles = Arc::new(SteamTitles::new(None, SteamWebApi::default(), Arc::clone(&app.store)));
        let recorder = Arc::new(SteamRecorder::new(titles, SteamWebApi::default(), Arc::clone(&app.store), Arc::clone(&app.settings)));
        let secrets = Arc::new(EphemeralSecretStore::default());
        let importer = SteamWebImporter::new(
            SteamWebApi::default(),
            None,
            recorder,
            Arc::clone(&app.settings),
            Arc::clone(&secrets) as Arc<dyn SecretStore>,
            Arc::clone(&app.status),
        );

        importer.set_api_key("  ABCDEF  ");
        assert_eq!(secrets.load(API_KEY_ACCOUNT).as_deref(), Some("ABCDEF"));
        assert!(!std::fs::read_to_string(directory.path().join("settings.json")).unwrap_or_default().contains("ABCDEF"));

        importer.set_api_key("");
        assert_eq!(importer.api_key(), "");
    }
}
