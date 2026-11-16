use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use prisma_core::domain::{Game, GameId, Platform};
use prisma_core::steam::{self, SteamAchievement, SteamWebApi};

use super::steam_titles::SteamTitles;
use crate::settings::SettingsStore;
use crate::store::LibraryStore;

/// Transforme les succès Steam, d'où qu'ils viennent, en jeu de la bibliothèque.
pub struct SteamRecorder {
    pub titles: Arc<SteamTitles>,
    api: SteamWebApi,
    store: Arc<LibraryStore>,
    settings: Arc<SettingsStore>,
    pending_rarity: Mutex<HashSet<u32>>,
}

impl SteamRecorder {
    pub fn new(titles: Arc<SteamTitles>, api: SteamWebApi, store: Arc<LibraryStore>, settings: Arc<SettingsStore>) -> Self {
        SteamRecorder { titles, api, store, settings, pending_rarity: Mutex::default() }
    }

    pub fn record(self: &Arc<Self>, app_id: u32, title: &str, achievements: &[SteamAchievement], announcing: bool) -> Option<Game> {
        if achievements.is_empty() {
            return None;
        }
        let rarity = self.unlock_percentages(app_id);
        let game = steam::make_game(app_id, title, achievements, rarity.as_ref());
        self.store.record(game.clone(), announcing);
        self.fetch_rarity_if_missing(app_id);
        Some(game)
    }

    /// Recalcule les grades de tous les jeux Steam, par exemple après un changement de mode d'attribution.
    pub fn reapply_tiers(self: &Arc<Self>) {
        for app_id in self.store.stored_games_on(Platform::Steam).iter().filter_map(|game| game.id.steam_app_id()) {
            self.reapply_tiers_of(app_id);
        }
    }

    fn reapply_tiers_of(self: &Arc<Self>, app_id: u32) {
        let Some(game) = self.store.game(&GameId::steam(app_id)) else { return };
        let achievements: Vec<_> = game.regular_trophies().map(SteamAchievement::from).collect();
        self.record(app_id, &game.title, &achievements, false);
    }

    /// La rareté mondiale sert seulement à attribuer les grades ; elle n'est jamais affichée.
    fn unlock_percentages(&self, app_id: u32) -> Option<std::collections::HashMap<String, f64>> {
        if !self.settings.get().use_steam_rarity {
            return None;
        }
        self.store.source_cache().steam_rarity.remove(&app_id)
    }

    fn fetch_rarity_if_missing(self: &Arc<Self>, app_id: u32) {
        let is_known = self.store.source_cache().steam_rarity.contains_key(&app_id);
        if !self.settings.get().use_steam_rarity || is_known || !self.pending_rarity.lock().expect("raretés accessibles").insert(app_id) {
            return;
        }
        let recorder = Arc::clone(self);
        tauri::async_runtime::spawn(async move {
            let result = recorder.api.unlock_percentages(app_id).await;
            recorder.pending_rarity.lock().expect("raretés accessibles").remove(&app_id);
            match result {
                Ok(percentages) if !percentages.is_empty() => {
                    recorder.store.update_cache(|cache| {
                        cache.steam_rarity.insert(app_id, percentages);
                    });
                    recorder.reapply_tiers_of(app_id);
                }
                Ok(_) => {}
                Err(error) => log::warn!("Rareté Steam de {app_id} indisponible : {error}"),
            }
        });
    }
}
