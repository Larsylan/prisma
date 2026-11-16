// Pas de fenêtre de console sous Windows dans la version distribuée.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod environment;
mod events;
mod notifications;
mod secrets;
mod services;
mod settings;
mod store;
mod sync;
#[cfg(test)]
mod testing;
mod themes;
mod tray;
mod views;

use std::sync::Arc;

use tauri::{Listener, Manager, RunEvent, WindowEvent};
use tauri_plugin_autostart::MacosLauncher;

use crate::notifications::ToastWindow;
use crate::services::Services;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _arguments, _directory| tray::show_main_window(app)))
        .plugin(tauri_plugin_log::Builder::new().level(log::LevelFilter::Info).build())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // L'app vit dans la barre des menus : pas d'icône dans le Dock.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let handle = app.handle().clone();
            let services = Arc::new(Services::new(&handle));
            app.manage(Arc::clone(&services));
            ToastWindow::create(&handle)?;
            tray::create(&handle, &services)?;

            let refreshed = Arc::clone(&services);
            let tray_app = handle.clone();
            app.listen_any(events::LIBRARY_CHANGED, move |_| tray::refresh(&tray_app, &refreshed));

            services.start();
            Ok(())
        })
        .on_window_event(|window, event| {
            // Fermer la fenêtre ne quitte pas l'app : la détection des trophées continue.
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::library_overview,
            commands::game_detail,
            commands::set_trophy_earned,
            commands::set_trophy_tier,
            commands::remove_game,
            commands::switch_catalog,
            commands::add_switch_game,
            commands::add_custom_switch_game,
            commands::sync_all,
            commands::import_steam_history,
            commands::sync_playstation,
            commands::connect_playstation,
            commands::disconnect_playstation,
            commands::steam_api_key,
            commands::set_steam_api_key,
            commands::sync_status,
            commands::settings,
            commands::update_settings,
            commands::test_notification,
            commands::system_info,
            commands::set_launch_at_login,
            commands::theme_gallery,
            commands::active_theme,
        ])
        .build(tauri::generate_context!())
        .expect("démarrage de Prisma")
        .run(|app, event| {
            if let RunEvent::Exit = event {
                app.state::<Arc<Services>>().store.save_now();
            }
        });
}
