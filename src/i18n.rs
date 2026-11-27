use crate::state::AppLanguage;
use std::collections::HashMap;
use std::sync::OnceLock;

const ARABIC: &str = include_str!("locales/ar.toml");

static APP_LANGUAGE: OnceLock<AppLanguage> = OnceLock::new();

pub fn initialize(app_language: AppLanguage) {
    let _ = APP_LANGUAGE.set(app_language);
}

pub fn tr(text: &str) -> String {
    dictionary()
        .and_then(|dictionary| dictionary.get(text).cloned())
        .unwrap_or_else(|| text.to_owned())
}

pub fn tr_vars(text: &str, vars: &[(&str, &str)]) -> String {
    let mut text = tr(text);
    for (key, value) in vars {
        text = text.replace(&format!("{{{key}}}"), value);
    }
    text
}

pub fn is_rtl() -> bool {
    dictionary().is_some()
}

fn dictionary() -> Option<&'static HashMap<String, String>> {
    static DICTIONARY: OnceLock<Option<HashMap<String, String>>> = OnceLock::new();
    DICTIONARY
        .get_or_init(|| match language_code() {
            "ar" => toml::from_str::<HashMap<String, String>>(ARABIC).ok(),
            _ => None,
        })
        .as_ref()
}

fn language_code() -> &'static str {
    static LANGUAGE: OnceLock<String> = OnceLock::new();
    LANGUAGE.get_or_init(|| match current_app_language() {
        AppLanguage::Arabic => "ar".to_owned(),
        AppLanguage::English => "en".to_owned(),
        AppLanguage::System => system_language_code(),
    })
}

fn current_app_language() -> AppLanguage {
    APP_LANGUAGE.get().copied().unwrap_or_default()
}

fn system_language_code() -> String {
    gtk::glib::language_names()
        .first()
        .map(|name| name.as_str())
        .unwrap_or("en")
        .split(['-', '_'])
        .next()
        .unwrap_or("en")
        .to_lowercase()
}
