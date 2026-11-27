use super::Choice;
use gtk::glib;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, glib::Enum, Serialize, Deserialize)]
#[enum_type(name = "RovarchAppLanguage")]
#[serde(rename_all = "snake_case")]
pub enum AppLanguage {
    #[default]
    System,
    English,
    Arabic,
}

impl AppLanguage {
    pub fn is_system(language: &Self) -> bool {
        *language == Self::System
    }
}

impl Choice for AppLanguage {
    const ALL: &'static [Self] = &[Self::System, Self::English, Self::Arabic];

    fn label(self) -> &'static str {
        match self {
            Self::System => "System",
            Self::English => "English",
            Self::Arabic => "العربية",
        }
    }
}
