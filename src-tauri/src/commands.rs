//! Commandes appelées par l'interface.

use std::sync::Arc;

use prisma_core::domain::{GameId, TrophyRef, TrophyTier};
use prisma_core::switch;
use prisma_core::theme::ThemePalette;
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_autostart::ManagerExt;

use crate::events;
use crate::i18n::Locale;
use crate::services::Services;
use crate::settings::Settings;
use crate::sync::{Failure, SyncStatus};
use crate::themes::{self, ThemeOption};
use crate::tray;
use crate::views::{CatalogItem, GameDetail, LibraryOverview};

type Shared<'a> = State<'a, Arc<Services>>;

#[tauri::command]
pub fn library_overview(services: Shared<'_>) -> LibraryOverview {
    LibraryOverview::new(services.settings.get().profile_name, &services.store.entries())
}

#[tauri::command]
pub fn game_detail(services: Shared<'_>, id: String) -> Option<GameDetail> {
    services.store.game(&GameId::new(id)).map(|game| GameDetail::new(&game, &services.store))
}

#[tauri::command]
pub fn set_trophy_earned(services: Shared<'_>, reference: TrophyRef, earned: bool) {
    services.store.set_earned(earned, &reference);
}

#[tauri::command]
pub fn set_trophy_tier(services: Shared<'_>, reference: TrophyRef, tier: Option<TrophyTier>) {
    services.store.set_tier(tier, &reference);
}

#[tauri::command]
pub fn remove_game(services: Shared<'_>, id: String) {
    services.store.remove(&GameId::new(id));
}

#[tauri::command]
pub fn switch_catalog(services: Shared<'_>) -> Vec<CatalogItem> {
    let mut entries: Vec<_> = switch::entries().iter().collect();
    entries.sort_by_key(|entry| entry.title.to_lowercase());
    entries.into_iter().map(|entry| CatalogItem::new(entry, services.switch_library.contains(entry))).collect()
}

#[tauri::command]
pub fn add_switch_game(services: Shared<'_>, catalog_id: String) {
    if let Some(entry) = switch::entries().iter().find(|entry| entry.id == catalog_id) {
        services.switch_library.add(entry);
    }
}

#[tauri::command]
pub fn add_custom_switch_game(services: Shared<'_>, title: String) {
    let title = title.trim();
    if !title.is_empty() {
        services.switch_library.add(&switch::custom_entry(title));
    }
}

#[tauri::command]
pub async fn sync_all(services: Shared<'_>) -> Result<(), ()> {
    services.coordinator.sync_all().await;
    Ok(())
}

#[tauri::command]
pub async fn import_steam_history(services: Shared<'_>) -> Result<(), ()> {
    services.steam_web.import_history().await;
    Ok(())
}

#[tauri::command]
pub async fn sync_playstation(services: Shared<'_>) -> Result<(), ()> {
    services.playstation.sync(false).await;
    Ok(())
}

#[tauri::command]
pub async fn connect_playstation(services: Shared<'_>, npsso: String) -> Result<(), Failure> {
    services.playstation.connect(&npsso).await.map_err(|error| Failure::from(&error))
}

#[tauri::command]
pub async fn disconnect_playstation(services: Shared<'_>, removing_trophies: bool) -> Result<(), ()> {
    services.playstation.disconnect(removing_trophies).await;
    Ok(())
}

#[tauri::command]
pub fn steam_api_key(services: Shared<'_>) -> String {
    services.steam_web.api_key()
}

#[tauri::command]
pub fn set_steam_api_key(services: Shared<'_>, key: String) {
    services.steam_web.set_api_key(&key);
}

#[tauri::command]
pub fn sync_status(services: Shared<'_>) -> SyncStatus {
    services.status.current()
}

#[tauri::command]
pub fn settings(services: Shared<'_>) -> Settings {
    services.settings.get()
}

/// Enregistre les réglages et applique ceux qui modifient le comportement des sources.
#[tauri::command]
pub fn update_settings(app: AppHandle, services: Shared<'_>, settings: Settings) {
    let previous = services.settings.replace(settings.clone());
    if previous.grouping_mode != settings.grouping_mode || previous.profile_name != settings.profile_name {
        services.store.regroup();
    }
    if previous.use_steam_rarity != settings.use_steam_rarity {
        services.steam_recorder.reapply_tiers();
    }
    if previous.playstation_polling != settings.playstation_polling {
        services.playstation.update_polling();
    }
    if previous.theme != settings.theme {
        if let Err(error) = app.emit(events::THEME_CHANGED, settings.theme.palette()) {
            log::error!("Changement de thème impossible : {error}");
        }
    }
    if previous.language != settings.language {
        tray::refresh(&app, &services);
        if let Err(error) = app.emit(events::LANGUAGE_CHANGED, settings.language.locale()) {
            log::error!("Changement de langue impossible : {error}");
        }
    }
}

#[tauri::command]
pub fn locale(services: Shared<'_>) -> Locale {
    services.settings.get().language.locale()
}

#[tauri::command]
pub fn theme_gallery() -> Vec<ThemeOption> {
    themes::options()
}

#[tauri::command]
pub fn active_theme(services: Shared<'_>) -> ThemePalette {
    services.settings.get().theme.palette()
}

#[tauri::command]
pub fn test_notification(services: Shared<'_>, tier: Option<TrophyTier>) {
    services.notifier.announce_test(tier, services.settings.get().language.locale().texts());
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    data_directory: String,
    detected_steam_id64: Option<String>,
    launch_at_login: bool,
}

#[tauri::command]
pub fn system_info(app: AppHandle, services: Shared<'_>) -> SystemInfo {
    SystemInfo {
        data_directory: services.environment.data_directory.display().to_string(),
        detected_steam_id64: services.steam_web.detected_steam_id64(),
        launch_at_login: app.autolaunch().is_enabled().unwrap_or(false),
    }
}

#[tauri::command]
pub fn set_launch_at_login(app: AppHandle, enabled: bool) -> Result<(), String> {
    let launcher = app.autolaunch();
    let result = if enabled { launcher.enable() } else { launcher.disable() };
    result.map_err(|error| error.to_string())
}
