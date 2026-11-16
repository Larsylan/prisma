use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use prisma_core::domain::GameId;
use prisma_core::steam::{SteamClientCache, SteamWebApi, title};

use crate::store::LibraryStore;

/// Le nom présent dans la liste de succès est souvent un nom de code interne : le nom officiel est préféré.
pub struct SteamTitles {
    steam_cache: Option<SteamClientCache>,
    api: SteamWebApi,
    store: Arc<LibraryStore>,
    /// Une recherche infructueuse n'est pas retentée avant le prochain lancement.
    attempted_lookups: Mutex<HashSet<u32>>,
}

impl SteamTitles {
    pub fn new(steam_cache: Option<SteamClientCache>, api: SteamWebApi, store: Arc<LibraryStore>) -> Self {
        SteamTitles { steam_cache, api, store, attempted_lookups: Mutex::default() }
    }

    /// Titre à afficher tout de suite ; le nom de la boutique, s'il manque, est recherché en arrière-plan.
    pub fn title(self: &Arc<Self>, app_id: u32, schema_name: &str) -> String {
        if let Some(known) = self.store.source_cache().steam_titles.get(&app_id) {
            return known.clone();
        }
        if let Some(installed) = self.steam_cache.as_ref().and_then(|cache| cache.installed_game_names().remove(&app_id)) {
            self.remember(&installed, app_id);
            return installed;
        }
        self.look_up_store_title(app_id);
        if title::is_meaningful(schema_name) { schema_name.to_owned() } else { title::placeholder(app_id) }
    }

    pub fn remember(&self, title: &str, app_id: u32) {
        self.store.update_cache(|cache| {
            cache.steam_titles.insert(app_id, title.to_owned());
        });
    }

    fn look_up_store_title(self: &Arc<Self>, app_id: u32) {
        if !self.attempted_lookups.lock().expect("recherches accessibles").insert(app_id) {
            return;
        }
        let titles = Arc::clone(self);
        tauri::async_runtime::spawn(async move {
            let Some(title) = titles.api.store_title(app_id).await else { return };
            titles.remember(&title, app_id);
            if let Some(mut game) = titles.store.game(&GameId::steam(app_id)) {
                game.rename(&title);
                titles.store.record(game, false);
            }
        });
    }
}
