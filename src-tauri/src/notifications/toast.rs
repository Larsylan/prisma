use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

use super::Announcement;
use crate::events;
use crate::settings::ToastPosition;

pub const LABEL: &str = "toast";
/// La carte mesure 420 × 100 ; la marge transparente laisse place à son ombre.
const WIDTH: f64 = 440.0;
const HEIGHT: f64 = 120.0;
const SCREEN_MARGIN: f64 = 18.0;
/// Durée de l'animation de disparition jouée par la page de la carte.
const FADE_OUT: Duration = Duration::from_millis(350);

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ToastPayload<'a> {
    #[serde(flatten)]
    announcement: &'a Announcement,
    /// La carte glisse depuis le bord de l'écran le plus proche.
    slides_from_left: bool,
}

/// Fenêtre de la carte « Vous avez obtenu un trophée », affichée par-dessus tout sans jamais
/// prendre le focus du clavier ni de la souris.
pub struct ToastWindow {
    app: AppHandle,
}

impl ToastWindow {
    pub fn new(app: AppHandle) -> Self {
        ToastWindow { app }
    }

    pub fn create(app: &AppHandle) -> tauri::Result<WebviewWindow> {
        let window = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("toast.html".into()))
            .title("Prisma")
            .inner_size(WIDTH, HEIGHT)
            .resizable(false)
            .decorations(false)
            .transparent(true)
            .shadow(false)
            .always_on_top(true)
            .visible_on_all_workspaces(true)
            .skip_taskbar(true)
            .focused(false)
            .focusable(false)
            .visible(false)
            .build()?;
        platform::prepare(&window)?;
        Ok(window)
    }

    pub async fn present(&self, announcement: &Announcement, duration: Duration, position: ToastPosition) {
        let Some(window) = self.app.get_webview_window(LABEL) else { return };
        let payload = ToastPayload { announcement, slides_from_left: position == ToastPosition::TopLeft };
        if let Err(error) = window.emit(events::TOAST_SHOW, payload) {
            log::error!("Notification impossible : {error}");
            return;
        }
        let (app, shown) = (self.app.clone(), window.clone());
        // Écrans, pointeur et fenêtre ne se manipulent que depuis le fil principal : sous Linux, Xlib s'interrompt sinon.
        let _ = self.app.run_on_main_thread(move || {
            if let Err(error) = place(&app, &shown, position) {
                log::warn!("Placement de la notification impossible : {error}");
            }
            platform::show(&shown);
        });
        tokio::time::sleep(duration).await;
        let _ = window.emit(events::TOAST_HIDE, ());
        tokio::time::sleep(FADE_OUT).await;
        let _ = window.hide();
    }
}

/// En haut de l'écran où se trouve le pointeur.
fn place(app: &AppHandle, window: &WebviewWindow, position: ToastPosition) -> tauri::Result<()> {
    let pointer = app.cursor_position()?;
    let monitor = match app.monitor_from_point(pointer.x, pointer.y)? {
        Some(monitor) => monitor,
        None => match app.primary_monitor()? {
            Some(monitor) => monitor,
            None => return Ok(()),
        },
    };
    let scale = monitor.scale_factor();
    let area = monitor.work_area();
    let margin = (SCREEN_MARGIN * scale) as i32;
    let width = (WIDTH * scale) as i32;
    let x = match position {
        ToastPosition::TopLeft => area.position.x + margin,
        ToastPosition::TopRight => area.position.x + area.size.width as i32 - width - margin,
    };
    window.set_position(PhysicalPosition::new(x, area.position.y + margin))
}

#[cfg(target_os = "macos")]
mod platform {
    use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior};
    use objc2_core_graphics::CGShieldingWindowLevel;
    use tauri::WebviewWindow;

    /// Le niveau « bouclier » passe au-dessus des jeux en plein écran, sur tous les bureaux.
    pub fn prepare(window: &WebviewWindow) -> tauri::Result<()> {
        window.set_ignore_cursor_events(true)?;
        with_ns_window(window, |ns_window| {
            ns_window.setLevel(CGShieldingWindowLevel() as isize);
            ns_window.setCollectionBehavior(
                NSWindowCollectionBehavior::CanJoinAllSpaces
                    | NSWindowCollectionBehavior::FullScreenAuxiliary
                    | NSWindowCollectionBehavior::Stationary
                    | NSWindowCollectionBehavior::IgnoresCycle,
            );
        });
        Ok(())
    }

    /// `orderFrontRegardless` affiche la fenêtre sans activer l'app : le jeu garde le focus.
    pub fn show(window: &WebviewWindow) {
        with_ns_window(window, |ns_window| ns_window.orderFrontRegardless());
    }

    fn with_ns_window(window: &WebviewWindow, action: impl FnOnce(&NSWindow)) {
        if let Ok(pointer) = window.ns_window() {
            // SAFETY : Tauri fournit un pointeur valide vers la NSWindow, utilisé sur le fil principal.
            action(unsafe { &*pointer.cast::<NSWindow>() });
        }
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use tauri::WebviewWindow;
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GWL_EXSTYLE, GetWindowLongPtrW, HWND_TOPMOST, SW_SHOWNOACTIVATE, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SetWindowLongPtrW,
        SetWindowPos, ShowWindow, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT,
    };

    /// Fenêtre outil, transparente aux clics, qui ne s'active jamais.
    pub fn prepare(window: &WebviewWindow) -> tauri::Result<()> {
        window.set_ignore_cursor_events(true)?;
        if let Some(hwnd) = handle(window) {
            // SAFETY : la poignée vient de Tauri et désigne une fenêtre vivante.
            unsafe {
                let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                let extra = (WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_TRANSPARENT) as isize;
                SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style | extra);
            }
        }
        Ok(())
    }

    pub fn show(window: &WebviewWindow) {
        if let Some(hwnd) = handle(window) {
            // SAFETY : la poignée vient de Tauri et désigne une fenêtre vivante.
            unsafe {
                ShowWindow(hwnd, SW_SHOWNOACTIVATE);
                SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
            }
        }
    }

    fn handle(window: &WebviewWindow) -> Option<HWND> {
        window.hwnd().ok().map(|hwnd| hwnd.0 as HWND)
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod platform {
    use tauri::WebviewWindow;

    pub fn prepare(_window: &WebviewWindow) -> tauri::Result<()> {
        Ok(())
    }

    /// Sous Wayland, certains bureaux (GNOME notamment) refusent qu'une fenêtre passe au-dessus d'un jeu en plein écran.
    /// GTK ne laisse passer les clics qu'à travers une fenêtre déjà affichée : avant, elle n'existe pas encore à l'écran.
    pub fn show(window: &WebviewWindow) {
        if let Err(error) = window.show().and_then(|()| window.set_ignore_cursor_events(true)) {
            log::error!("Affichage de la notification impossible : {error}");
        }
    }
}
