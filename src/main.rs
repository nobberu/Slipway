use std::ffi::c_void;
use std::time::Duration;

slint::include_modules!();

const CATALOG: &[(&str, &str, &str)] = &[
    ("App 1", "Windows", "Application"),
    ("App 2", "Windows", "Application"),
    ("App 3", "Windows", "Application"),
    ("App 4", "Windows", "Application"),
    ("App 5", "Windows", "Application"),
    ("Command 1", "System Actions", "Command"),
    ("Command 2", "System Actions", "Command"),
    ("Command 3", "System Actions", "Command"),
    ("Command 4", "System Actions", "Command"),
    ("Command 4", "System Actions", "Command"),
    ("Command 4", "System Actions", "Command"),
    ("Command 4", "System Actions", "Command"),
];

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

    let mut results: Vec<ResultItem> = Vec::new();
    let mut current_kind: Option<&str> = None;

    for (title, subtitle, kind) in CATALOG {
        if current_kind != Some(*kind) {
            current_kind = Some(*kind);
            results.push(ResultItem {
                title: format!("{kind}s").into(),
                subtitle: "".into(),
                kind: "".into(),
                initial: "".into(),
                is_header: true,
            });
        }

        results.push(ResultItem {
            title: (*title).into(),
            subtitle: (*subtitle).into(),
            kind: (*kind).into(),
            initial: title
                .chars()
                .next()
                .unwrap_or('?')
                .to_uppercase()
                .to_string()
                .into(),
            is_header: false,
        });
    }

    ui.set_results(std::rc::Rc::new(slint::VecModel::from(results)).into());
    ui.set_selected(1);

    let weak = ui.as_weak();
    slint::Timer::single_shot(Duration::from_millis(50), move || {
        if let Some(ui) = weak.upgrade() {
            apply_blur(&ui);
        }
    });

    ui.run()
}