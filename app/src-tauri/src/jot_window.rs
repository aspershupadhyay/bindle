// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! The Jot down window and its system-wide shortcut (handbook P6 and P13).
//!
//! Alt+Shift+J opens a small window over whatever the person is doing. They
//! type, press Enter, see "Saved", and the window gets out of the way.

use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

/// The label of the Jot down window in `tauri.conf.json`.
pub const LABEL: &str = "jot";

fn shortcut() -> Shortcut {
    Shortcut::new(Some(Modifiers::ALT | Modifiers::SHIFT), Code::KeyJ)
}

/// Adds the global shortcut plugin and claims Alt+Shift+J.
///
/// Some desktops (Wayland sessions, or another app holding the keys) refuse a
/// global shortcut. That is not fatal: the shortcut still works inside Bindle,
/// and the Jot down button is always there.
pub fn register<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    app.plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(|app, pressed, event| {
                if pressed == &shortcut() && event.state() == ShortcutState::Pressed {
                    show(app);
                }
            })
            .build(),
    )?;
    if let Err(error) = app.global_shortcut().register(shortcut()) {
        eprintln!("Bindle: Alt+Shift+J could not be claimed system-wide: {error}");
    }
    Ok(())
}

/// Shows the Jot down window, centred and focused, with an empty field.
pub fn show<R: Runtime>(app: &AppHandle<R>) {
    let Some(window) = app.get_webview_window(LABEL) else {
        return;
    };
    // Each step can fail on an odd window manager; showing matters most.
    let _ = window.center();
    let _ = window.show();
    let _ = window.set_focus();
    let _ = window.emit_to(LABEL, "jot-opened", ());
}

/// Hides the Jot down window.
pub fn hide<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window(LABEL) {
        let _ = window.hide();
    }
}
