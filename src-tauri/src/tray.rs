//! Icône dans la barre des menus (macOS) ou la zone de notification (Windows, Linux).

use std::sync::Arc;

use prisma_core::domain::{TrophyStatistics, TrophyTier};
use tauri::image::Image;
use tauri::menu::{IsMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::{AppHandle, Emitter, Manager, Wry};

use crate::events;
use crate::services::Services;

const RECENT_TROPHY_COUNT: usize = 5;
const OPEN: &str = "open";
const SETTINGS: &str = "settings";
const TEST: &str = "test";
const SYNC: &str = "sync";
const QUIT: &str = "quit";

pub fn create(app: &AppHandle, services: &Arc<Services>) -> tauri::Result<TrayIcon> {
    let tray = TrayIconBuilder::with_id("prisma")
        .icon(icon()?)
        .icon_as_template(cfg!(target_os = "macos"))
        .tooltip("Prisma")
        .menu(&menu(app, services)?)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| handle(app, event.id().as_ref()))
        .build(app)?;
    Ok(tray)
}

/// Le menu affiche le niveau et les derniers trophées : il est reconstruit à chaque modification.
pub fn refresh(app: &AppHandle, services: &Arc<Services>) {
    let Some(tray) = app.tray_by_id("prisma") else { return };
    if let Err(error) = menu(app, services).and_then(|menu| tray.set_menu(Some(menu))) {
        log::error!("Mise à jour du menu impossible : {error}");
    }
}

/// Sur macOS, une silhouette que le système colore selon le thème ; ailleurs, l'icône de l'app.
fn icon() -> tauri::Result<Image<'static>> {
    let bytes: &[u8] =
        if cfg!(target_os = "macos") { include_bytes!("../icons/tray@2x.png") } else { include_bytes!("../icons/32x32.png") };
    Image::from_bytes(bytes)
}

fn menu(app: &AppHandle, services: &Arc<Services>) -> tauri::Result<Menu<Wry>> {
    let texts = services.settings.get().language.locale().texts();
    let games = services.store.entries();
    let summary = format!("{} {} · {} {}", texts.level, games.level().value, games.earned_count(TrophyTier::Platinum), texts.platinums);
    let mut items: Vec<Box<dyn IsMenuItem<Wry>>> = vec![Box::new(disabled(app, &summary)?), Box::new(PredefinedMenuItem::separator(app)?)];
    for earned in games.recently_earned(RECENT_TROPHY_COUNT) {
        let line = format!("{} — {} ({})", texts.tier(earned.trophy.tier), earned.trophy.name, earned.game.title);
        items.push(Box::new(disabled(app, &line)?));
    }
    items.push(Box::new(PredefinedMenuItem::separator(app)?));
    items.push(Box::new(MenuItem::with_id(app, OPEN, texts.open, true, Some("CmdOrCtrl+O"))?));
    items.push(Box::new(MenuItem::with_id(app, SETTINGS, texts.settings, true, Some("CmdOrCtrl+,"))?));
    items.push(Box::new(MenuItem::with_id(app, TEST, texts.test_notification, true, None::<&str>)?));
    items.push(Box::new(MenuItem::with_id(app, SYNC, texts.sync, true, None::<&str>)?));
    items.push(Box::new(PredefinedMenuItem::separator(app)?));
    items.push(Box::new(MenuItem::with_id(app, QUIT, texts.quit, true, Some("CmdOrCtrl+Q"))?));
    let references: Vec<&dyn IsMenuItem<Wry>> = items.iter().map(|item| item.as_ref()).collect();
    Menu::with_items(app, &references)
}

fn disabled(app: &AppHandle, text: &str) -> tauri::Result<MenuItem<Wry>> {
    MenuItem::new(app, text, false, None::<&str>)
}

fn handle(app: &AppHandle, id: &str) {
    let services = app.state::<Arc<Services>>();
    match id {
        OPEN => show_main_window(app),
        SETTINGS => {
            show_main_window(app);
            let _ = app.emit(events::NAVIGATE, "settings");
        }
        TEST => services.notifier.announce_test(None, services.settings.get().language.locale().texts()),
        SYNC => {
            let services = Arc::clone(&services);
            tauri::async_runtime::spawn(async move { services.coordinator.sync_all().await });
        }
        QUIT => app.exit(0),
        _ => {}
    }
}

pub fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}
