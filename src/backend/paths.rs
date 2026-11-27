use gtk::glib;
use std::fs;
use std::path::{Path, PathBuf};

const LONGEST_LINK_CHAIN: usize = 16;

pub fn rovarch_data_directory() -> PathBuf {
    rovarch_directory_of(glib::user_data_dir())
}

pub fn rovarch_rosemary_versions_directory() -> PathBuf {
    rovarch_data_directory().join("rosemary")
}

pub fn rovarch_rosemary_directory(version: &str) -> PathBuf {
    rovarch_rosemary_versions_directory().join(version)
}

pub fn rovarch_prefix_directory() -> PathBuf {
    rovarch_data_directory().join("prefix")
}

pub fn roblox_directory() -> PathBuf {
    rovarch_data_directory().join("roblox")
}

pub fn studio_directory() -> PathBuf {
    rovarch_data_directory().join("studio")
}

pub fn applications_directory() -> PathBuf {
    glib::user_data_dir().join("applications")
}

pub fn mime_directory() -> PathBuf {
    glib::user_data_dir().join("mime")
}

pub fn mime_defaults_file() -> PathBuf {
    glib::user_config_dir().join("mimeapps.list")
}

fn rovarch_directory_of(base_directory: PathBuf) -> PathBuf {
    base_directory.join("rovarch")
}

pub fn resolved_on_host_os(path: &Path) -> Option<PathBuf> {
    if !path.is_absolute() {
        return path.canonicalize().ok();
    }
    let mut path = path.to_owned();
    for _ in 0..LONGEST_LINK_CHAIN {
        let Ok(link_target) = fs::read_link(&path) else {
            return path.canonicalize().ok();
        };
        path = path.parent()?.join(link_target);
    }
    None
}

pub fn desktop_entry_icon_file() -> PathBuf {
    rovarch_data_directory()
        .join("icons")
        .join("io.rovarch.app.png")
}

pub fn legacy_entry_icon_file() -> PathBuf {
    rovarch_data_directory()
        .join("icons")
        .join("io.github.dratiux.Rovarch.png")
}

pub fn fast_flags_file() -> PathBuf {
    rovarch_data_directory()
        .join("ClientSettings")
        .join("ClientAppSettings.json")
}

pub fn logs_directory() -> PathBuf {
    rovarch_data_directory().join("logs")
}

pub fn downloads_directory() -> PathBuf {
    rovarch_directory_of(glib::user_cache_dir()).join("downloads")
}

pub fn butter_runner_directories() -> [PathBuf; 2] {
    [
        glib::user_data_dir().join("butter").join("runners"),
        glib::home_dir().join(".var/app/io.github.microlift.butter/data/butter/runners"),
    ]
}

pub fn server_history_file() -> PathBuf {
    rovarch_directory_of(glib::user_config_dir()).join("server-history.json")
}

pub fn regions_cache_file() -> PathBuf {
    rovarch_directory_of(glib::user_cache_dir()).join("rovalra-regions.json")
}

pub fn game_icons_directory() -> PathBuf {
    rovarch_directory_of(glib::user_cache_dir()).join("game-icons")
}

pub fn vr_directory() -> PathBuf {
    rovarch_directory_of(glib::user_cache_dir()).join("vr")
}

pub fn mods_directory() -> PathBuf {
    rovarch_data_directory().join("modifications")
}

pub fn mods_backup_directory() -> PathBuf {
    rovarch_data_directory().join("mods-backup")
}

pub fn mod_guard_file() -> PathBuf {
    rovarch_directory_of(glib::user_config_dir()).join("mod-guard.json")
}

pub fn fast_flag_allowlist_file() -> PathBuf {
    rovarch_directory_of(glib::user_cache_dir()).join("fast-flag-allowlist.json")
}
