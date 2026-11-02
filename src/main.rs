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
];

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    let results: Vec<ResultItem> = CATALOG
        .iter()
        .map(|(title, subtitle, kind)| ResultItem {
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
        })
        .collect();

    ui.set_results(std::rc::Rc::new(slint::VecModel::from(results)).into());

    ui.run()
}
