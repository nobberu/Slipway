// Copyright (C) 2026 nobberu <itsnobberu@gmail.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashSet;
use std::ffi::c_void;
use std::time::Duration;

#[cfg(target_os = "windows")]
use windows::{
    Win32::System::Com::{
        COINIT_APARTMENTTHREADED, CoInitializeEx, CoTaskMemFree, CoUninitialize, IBindCtx,
    },
    Win32::UI::Shell::{
        BHID_EnumItems, IEnumShellItems, IShellItem, SHCreateItemFromParsingName, SIGDN,
        SIGDN_DESKTOPABSOLUTEPARSING, SIGDN_NORMALDISPLAY,
    },
    core::HSTRING,
};

slint::include_modules!();

const COMMANDS: &[&str] = &[
    "Lock Screen",
    "Sleep",
    "Restart",
    "Shut Down",
    "Clipboard",
    "Confetti",
];

struct AppEntry {
    name: String,
    subtitle: String,
    kind: &'static str,
}

fn classify(app_id: &str) -> (&'static str, &'static str) {
    let id = app_id.to_lowercase();
    let launchers = [
        ("steam://", "Steam"),
        ("com.epicgames.launcher://", "Epic Games"),
        ("uplay://", "Ubisoft Connect"),
        ("goggalaxy://", "GOG Galaxy"),
        ("battlenet://", "Battle.net"),
        ("origin://", "EA"),
        ("origin2://", "EA"),
    ];
    for (needle, name) in launchers {
        if id.contains(needle) {
            return ("Game", name);
        }
    }
    ("Application", "Windows")
}

#[cfg(target_os = "windows")]
struct ComGuard;

#[cfg(target_os = "windows")]
impl ComGuard {
    fn new() -> Self {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        }
        ComGuard
    }
}

#[cfg(target_os = "windows")]
impl Drop for ComGuard {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}

#[cfg(target_os = "windows")]
fn item_name(item: &IShellItem, kind: SIGDN) -> Option<String> {
    unsafe {
        let p = item.GetDisplayName(kind).ok()?;
        let s = p.to_string().ok();
        CoTaskMemFree(Some(p.0 as *const c_void));
        s
    }
}

#[cfg(target_os = "windows")]
fn scan_apps() -> windows::core::Result<Vec<AppEntry>> {
    let _com = ComGuard::new();

    unsafe {
        let folder: IShellItem =
            SHCreateItemFromParsingName(&HSTRING::from("shell:AppsFolder"), None::<&IBindCtx>)?;
        let items: IEnumShellItems = folder.BindToHandler(None::<&IBindCtx>, &BHID_EnumItems)?;

        let mut seen = HashSet::new();
        let mut apps = Vec::new();

        loop {
            let mut batch: [Option<IShellItem>; 1] = [None];
            let mut fetched = 0u32;
            if items.Next(&mut batch, Some(&mut fetched)).is_err() || fetched == 0 {
                break;
            }
            let Some(item) = batch[0].take() else {
                break;
            };

            let Some(name) = item_name(&item, SIGDN_NORMALDISPLAY) else {
                continue;
            };
            let Some(parsing) = item_name(&item, SIGDN_DESKTOPABSOLUTEPARSING) else {
                continue;
            };

            let name = name.trim();
            if name.is_empty()
                || name.to_lowercase().contains("uninstall")
                || !seen.insert(name.to_lowercase())
            {
                continue;
            }

            let (kind, subtitle) = classify(&parsing);
            apps.push(AppEntry {
                name: name.to_string(),
                subtitle: subtitle.to_string(),
                kind,
            });
        }

        apps.sort_by_key(|a| a.name.to_lowercase());
        Ok(apps)
    }
}

#[cfg(target_os = "windows")]
fn load_apps() -> Vec<AppEntry> {
    scan_apps().unwrap()
}

fn header(title: &str) -> ResultItem {
    ResultItem {
        title: title.into(),
        subtitle: "".into(),
        kind: "".into(),
        initial: "".into(),
        is_header: true,
    }
}

fn row(title: &str, subtitle: &str, kind: &str) -> ResultItem {
    ResultItem {
        title: title.into(),
        subtitle: subtitle.into(),
        kind: kind.into(),
        initial: title
            .chars()
            .next()
            .unwrap_or('?')
            .to_uppercase()
            .to_string()
            .into(),
        is_header: false,
    }
}

fn build_results(apps: &[AppEntry]) -> Vec<ResultItem> {
    let mut results: Vec<ResultItem> = Vec::new();

    for (kind, title) in [("Application", "Applications"), ("Game", "Games")] {
        let group: Vec<&AppEntry> = apps.iter().filter(|a| a.kind == kind).collect();
        if group.is_empty() {
            continue;
        }
        results.push(header(title));
        for a in group {
            results.push(row(&a.name, &a.subtitle, a.kind));
        }
    }

    results.push(header("Commands"));
    for name in COMMANDS {
        results.push(row(name, "System Actions", "Command"));
    }

    results
}

#[cfg(target_os = "windows")]
#[link(name = "dwmapi")]
unsafe extern "system" {
    fn DwmSetWindowAttribute(
        hwnd: *mut c_void,
        attribute: u32,
        value: *const c_void,
        size: u32,
    ) -> i32;
}

#[cfg(target_os = "windows")]
fn round_corners(ui: &AppWindow) {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use slint::ComponentHandle;

    const DWMWA_WINDOW_CORNER_PREFERENCE: u32 = 33;
    const DWMWCP_ROUND: u32 = 2;

    let handle = ui.window().window_handle();

    if let Ok(wh) = handle.window_handle() {
        if let RawWindowHandle::Win32(h) = wh.as_raw() {
            let hwnd = h.hwnd.get() as *mut c_void;
            let pref = DWMWCP_ROUND;
            unsafe {
                DwmSetWindowAttribute(
                    hwnd,
                    DWMWA_WINDOW_CORNER_PREFERENCE,
                    &pref as *const u32 as *const c_void,
                    std::mem::size_of::<u32>() as u32,
                );
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn apply_blur(ui: &AppWindow) {
    use slint::ComponentHandle;
    round_corners(ui);
    let handle = ui.window().window_handle();
    let _ = window_vibrancy::apply_acrylic(&handle, None);
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    ui.set_results(std::rc::Rc::new(slint::VecModel::from(build_results(&[]))).into());
    ui.set_selected(1);

    {
        let weak = ui.as_weak();
        std::thread::spawn(move || {
            let started = std::time::Instant::now();
            let apps = load_apps();
            let games = apps.iter().filter(|a| a.kind == "Game").count();
            println!(
                "found {} apps ({} games) in {:?}",
                apps.len(),
                games,
                started.elapsed()
            );

            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = weak.upgrade() {
                    ui.set_results(
                        std::rc::Rc::new(slint::VecModel::from(build_results(&apps))).into(),
                    );
                    ui.set_selected(1);
                }
            });
        });
    }

    let weak = ui.as_weak();
    slint::Timer::single_shot(Duration::from_millis(50), move || {
        if let Some(ui) = weak.upgrade() {
            apply_blur(&ui);
        }
    });

    ui.run()
}
