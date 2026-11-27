use super::Choice;
use gtk::glib;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, glib::Enum, Serialize, Deserialize)]
#[enum_type(name = "RovarchLaunchMode")]
#[serde(rename_all = "snake_case")]
pub enum LaunchMode {
    #[default]
    OpenLauncher,
    LaunchRoblox,
}

impl Choice for LaunchMode {
    const ALL: &'static [Self] = &[Self::OpenLauncher, Self::LaunchRoblox];

    fn label(self) -> &'static str {
        match self {
            Self::OpenLauncher => "Open Rovarch",
            Self::LaunchRoblox => "Launch Roblox",
        }
    }
}
