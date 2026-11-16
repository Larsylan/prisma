use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::Local;
use prisma_core::domain::{Game, GameId};
use prisma_core::playstation::{PsnAuthenticator, PsnError, PsnTrophyClient, TitleSummary};
use tauri::async_runtime::JoinHandle;
use tokio::task::JoinSet;

use super::status::StatusBoard;
use crate::settings::SettingsStore;
use crate::store::LibraryStore;

const POLL_INTERVAL: Duration = Duration::from_secs(90);
const CONCURRENT_REQUESTS: usize = 4;

/// Importe les trophées PlayStation et vérifie régulièrement ceux gagnés sur console ou en Remote Play.
pub struct PlayStationSync {
    authenticator: Arc<PsnAuthenticator>,
    client: PsnTrophyClient,
    store: Arc<LibraryStore>,
    settings: Arc<SettingsStore>,
    status: Arc<StatusBoard>,
    is_syncing: AtomicBool,
    polling: Mutex<Option<JoinHandle<()>>>,
}

impl PlayStationSync {
    pub fn new(
        authenticator: Arc<PsnAuthenticator>,
        store: Arc<LibraryStore>,
        settings: Arc<SettingsStore>,
        status: Arc<StatusBoard>,
    ) -> Self {
        PlayStationSync {
            client: PsnTrophyClient::new(Arc::clone(&authenticator)),
            authenticator,
            store,
            settings,
            status,
            is_syncing: AtomicBool::new(false),
            polling: Mutex::default(),
        }
    }

    pub async fn start(self: &Arc<Self>) {
        let is_connected = self.authenticator.is_connected().await;
        self.status.update(|status| status.playstation_connected = is_connected);
        self.update_polling();
        self.sync(false).await;
    }

    pub async fn connect(&self, npsso: &str) -> Result<(), PsnError> {
        self.set_message("Connexion…");
        if let Err(error) = self.authenticator.connect(npsso).await {
            self.set_message(&error.to_string());
            return Err(error);
        }
        self.status.update(|status| {
            status.playstation_connected = true;
            status.playstation = "Connecté.".into();
        });
        self.sync(false).await;
        Ok(())
    }

    pub async fn disconnect(&self, removing_trophies: bool) {
        self.authenticator.disconnect().await;
        self.status.update(|status| {
            status.playstation_connected = false;
            status.playstation = "Déconnecté.".into();
        });
        if removing_trophies {
            self.store.remove_all_on(prisma_core::domain::Platform::Playstation);
        }
    }

    /// Ne relit que les jeux modifiés depuis la dernière synchronisation.
    pub async fn sync(&self, announcing: bool) {
        if !self.authenticator.is_connected().await || self.is_syncing.swap(true, Ordering::AcqRel) {
            return;
        }
        if !announcing {
            self.set_message("Synchronisation PlayStation…");
        }
        match self.sync_changed_titles(announcing).await {
            Ok(title_count) => {
                let time = Local::now().format("%H:%M");
                self.set_message(&format!("PlayStation synchronisé · {title_count} jeux · {time}"));
            }
            Err(error) => {
                let is_connected = self.authenticator.is_connected().await;
                self.status.update(|status| {
                    status.playstation = error.to_string();
                    status.playstation_connected = is_connected;
                });
            }
        }
        self.is_syncing.store(false, Ordering::Release);
    }

    pub fn update_polling(self: &Arc<Self>) {
        let mut polling = self.polling.lock().expect("vérification accessible");
        if let Some(task) = polling.take() {
            task.abort();
        }
        if !self.settings.get().playstation_polling {
            return;
        }
        let sync = Arc::clone(self);
        *polling = Some(tauri::async_runtime::spawn(async move {
            loop {
                tokio::time::sleep(POLL_INTERVAL).await;
                sync.sync(true).await;
            }
        }));
    }

    async fn sync_changed_titles(&self, announcing: bool) -> Result<usize, PsnError> {
        let titles = self.client.titles().await?;
        let changed: Vec<_> = titles.iter().filter(|title| self.needs_refresh(title)).cloned().collect();
        let mut updated = 0;
        for batch in changed.chunks(CONCURRENT_REQUESTS) {
            for (title, game) in self.fetch_games(batch).await {
                let Some(game) = game else { continue };
                self.store.record(game, announcing);
                let revision = title.last_updated_date_time.unwrap_or_default();
                self.store.update_cache(|cache| {
                    cache.playstation_revisions.insert(title.np_communication_id, revision);
                });
            }
            updated += batch.len();
            if !announcing {
                self.set_message(&format!("PlayStation : {updated}/{} jeux mis à jour…", changed.len()));
            }
        }
        Ok(titles.len())
    }

    fn needs_refresh(&self, title: &TitleSummary) -> bool {
        let known_revision = self.store.source_cache().playstation_revisions.get(&title.np_communication_id).cloned();
        let is_stored = self.store.game(&GameId::playstation(&title.np_communication_id)).is_some();
        known_revision != Some(title.last_updated_date_time.clone().unwrap_or_default()) || !is_stored
    }

    async fn fetch_games(&self, titles: &[TitleSummary]) -> Vec<(TitleSummary, Option<Game>)> {
        let mut requests = JoinSet::new();
        for title in titles.iter().cloned() {
            let client = self.client.clone();
            requests.spawn(async move {
                let game = client.game(&title).await.inspect_err(|error| log::warn!("{error}")).ok();
                (title, game)
            });
        }
        requests.join_all().await
    }

    fn set_message(&self, message: &str) {
        self.status.update(|status| status.playstation = message.to_owned());
    }
}

#[cfg(test)]
mod tests {
    use prisma_core::support::secrets::EphemeralSecretStore;

    use super::*;
    use crate::testing::TestApp;

    fn title(id: &str, revision: &str) -> TitleSummary {
        serde_json::from_value(serde_json::json!({
            "npServiceName": "trophy2",
            "npCommunicationId": id,
            "trophyTitleName": "Astro",
            "trophyTitlePlatform": "PS5",
            "lastUpdatedDateTime": revision,
        }))
        .unwrap()
    }

    #[test]
    fn only_new_or_updated_games_are_read_again() {
        let directory = tempfile::tempdir().unwrap();
        let app = TestApp::new(directory.path());
        let authenticator = Arc::new(PsnAuthenticator::new(Arc::new(EphemeralSecretStore::default())));
        let sync = PlayStationSync::new(authenticator, Arc::clone(&app.store), Arc::clone(&app.settings), Arc::clone(&app.status));
        app.store
            .record(Game::new(GameId::playstation("NPWR1"), prisma_core::domain::Platform::Playstation, "Astro", vec![], vec![]), false);
        app.store.update_cache(|cache| {
            cache.playstation_revisions.insert("NPWR1".into(), "2024-01-01".into());
        });

        assert!(!sync.needs_refresh(&title("NPWR1", "2024-01-01")));
        assert!(sync.needs_refresh(&title("NPWR1", "2024-02-01")));
        assert!(sync.needs_refresh(&title("NPWR2", "2024-01-01")));
    }
}
