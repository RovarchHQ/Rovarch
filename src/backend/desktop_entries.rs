use super::{Handler, handlers, mime_defaults, paths, updater};
use crate::utils::APP_ICON_PNG;
use anyhow::Context;
use std::fs;
use std::path::{Path, PathBuf};

const CURRENT_APP_ID: &str = "io.rovarch.app";
const LEGACY_APP_ID: &str = "io.github.dratiux.Rovarch";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Opens {
    Nothing,
    Link,
    File,
}

#[derive(Clone, Copy)]
pub struct DesktopEntry {
    pub file_name: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub command_line_option: Option<&'static str>,
    pub opens: Opens,
    pub mime_types: &'static [&'static str],
    pub shown_in_app_menu: bool,
}

fn current_launcher() -> anyhow::Result<PathBuf> {
    match updater::running_appimage() {
        Some(appimage) => Ok(appimage),
        None => std::env::current_exe().context("Couldn't find Rovarch's executable"),
    }
}

impl DesktopEntry {
    fn path(&self) -> PathBuf {
        paths::applications_directory().join(self.file_name)
    }

    pub fn exists(&self) -> bool {
        self.path().is_file()
    }

    pub fn add(&self) -> anyhow::Result<()> {
        self.write(&current_launcher()?)
    }

    pub fn refresh(&self) -> anyhow::Result<()> {
        if !self.exists() {
            return Ok(());
        }
        let launcher = current_launcher()?;
        let wanted = self.contents(&paths::desktop_entry_icon_file(), &launcher);
        let is_up_to_date = fs::read_to_string(self.path()).is_ok_and(|written| written == wanted);
        if is_up_to_date {
            return Ok(());
        }
        self.write(&launcher)
    }

    fn write(&self, launcher: &Path) -> anyhow::Result<()> {
        let icon = paths::desktop_entry_icon_file();
        if let Some(directory) = icon.parent() {
            fs::create_dir_all(directory)?;
        }
        fs::write(&icon, APP_ICON_PNG)
            .with_context(|| format!("Couldn't save the icon to {}", icon.display()))?;

        let path = self.path();
        if let Some(directory) = path.parent() {
            fs::create_dir_all(directory)?;
        }
        fs::write(&path, self.contents(&icon, launcher))
            .with_context(|| format!("Couldn't add {}", path.display()))
    }

    pub fn remove(&self) -> anyhow::Result<()> {
        let path = self.path();
        match fs::remove_file(&path) {
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
                Err(error).with_context(|| format!("Couldn't remove {}", path.display()))
            }
            _ => Ok(()),
        }
    }

    fn contents(&self, icon: &Path, launcher: &Path) -> String {
        let mut contents = format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Name={}\n\
             Comment={}\n\
             Exec={}\n\
             Icon={}\n\
             Terminal=false\n\
             Categories=Game;\n\
             StartupNotify=true\n",
            self.name,
            self.description,
            self.exec(launcher),
            icon.display()
        );
        if !self.mime_types.is_empty() {
            contents.push_str(&format!("MimeType={};\n", self.mime_types.join(";")));
        }
        if !self.shown_in_app_menu {
            contents.push_str("NoDisplay=true\n");
        }
        contents
    }

    fn exec(&self, launcher: &Path) -> String {
        let mut exec = quoted_for_exec(&launcher.display().to_string());

        if let Some(option) = self.command_line_option {
            exec.push(' ');
            exec.push_str(option);
        }

        match self.opens {
            Opens::Nothing => {}
            Opens::Link => exec.push_str(" %u"),
            Opens::File => exec.push_str(" %f"),
        }

        exec
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Shortcut {
    Rovarch,
    Roblox,
    Studio,
}

impl Shortcut {
    pub const ALL: [Self; 3] = [Self::Rovarch, Self::Roblox, Self::Studio];

    pub fn entry(self) -> DesktopEntry {
        let (file_name, name, description, command_line_option) = match self {
            Self::Rovarch => (
                "io.rovarch.app.desktop",
                "Rovarch",
                "Opens the launcher",
                None,
            ),
            Self::Roblox => (
                "io.rovarch.app.Roblox.desktop",
                "Rovarch Play",
                "Starts Rovarch Play without opening the launcher",
                Some("--roblox"),
            ),
            Self::Studio => (
                "io.rovarch.app.Studio.desktop",
                "Rovarch Studio",
                "Starts Rovarch Studio without opening the launcher",
                Some("--studio"),
            ),
        };

        DesktopEntry {
            file_name,
            name,
            description,
            command_line_option,
            opens: Opens::Nothing,
            mime_types: &[],
            shown_in_app_menu: true,
        }
    }
}

/// Moves what a pre-`io.rovarch.app` build installed under the old
/// application ID onto the new one: desktop entries, their default-app
/// associations, the icon, and the place file mime package.
///
/// Running it again is harmless; it does nothing once the old names are
/// gone.
pub fn retire_legacy_entries() -> anyhow::Result<()> {
    for shortcut in Shortcut::ALL {
        migrate_entry(&shortcut.entry())?;
    }
    for handler in Handler::ALL {
        migrate_entry(&handler.entry())?;
    }

    migrate_file(
        &paths::legacy_entry_icon_file(),
        &paths::desktop_entry_icon_file(),
        APP_ICON_PNG,
    )?;

    let package = handlers::place_mime_types_path();
    let legacy_package = package.with_file_name(format!("{LEGACY_APP_ID}.xml"));
    if legacy_package.is_file() {
        migrate_file(
            &legacy_package,
            &package,
            handlers::PLACE_MIME_TYPES.as_bytes(),
        )?;
        handlers::update_mime_database()?;
    }

    Ok(())
}

/// Rewrites an entry installed under the old file name, then removes the
/// old file. The rewrite matters: the new build points its icon at a
/// different path.
fn migrate_entry(entry: &DesktopEntry) -> anyhow::Result<()> {
    migrate_desktop_file(entry)?;
    migrate_associations(entry)
}

fn migrate_desktop_file(entry: &DesktopEntry) -> anyhow::Result<()> {
    let legacy = paths::applications_directory().join(legacy_file_name(entry.file_name));
    if !legacy.is_file() {
        return Ok(());
    }

    entry.add()?;
    fs::remove_file(&legacy).with_context(|| format!("Couldn't remove {}", legacy.display()))
}

/// Re-points the `mimeapps.list` associations from the old desktop file
/// name onto the new one. When the entry is installed, it goes back to
/// being the default, which is how `handlers::enable` left it.
fn migrate_associations(entry: &DesktopEntry) -> anyhow::Result<()> {
    if entry.mime_types.is_empty() {
        return Ok(());
    }

    let legacy_name = legacy_file_name(entry.file_name);
    let Ok(defaults) = fs::read_to_string(paths::mime_defaults_file()) else {
        return Ok(());
    };
    if !defaults.contains(&legacy_name) {
        return Ok(());
    }

    mime_defaults::remove_desktop_associations(&[legacy_name.as_str()])?;
    if entry.exists() {
        mime_defaults::make_default(entry.file_name, entry.mime_types)?;
    }
    Ok(())
}

fn legacy_file_name(file_name: &str) -> String {
    file_name.replacen(CURRENT_APP_ID, LEGACY_APP_ID, 1)
}

fn migrate_file(legacy: &Path, current: &Path, contents: &[u8]) -> anyhow::Result<()> {
    if !legacy.is_file() {
        return Ok(());
    }
    if let Some(directory) = current.parent() {
        fs::create_dir_all(directory)
            .with_context(|| format!("Couldn't create {}", directory.display()))?;
    }
    fs::write(current, contents)
        .with_context(|| format!("Couldn't write {}", current.display()))?;
    fs::remove_file(legacy).with_context(|| format!("Couldn't remove {}", legacy.display()))
}

fn quoted_for_exec(argument: &str) -> String {
    let argument = argument.replace('%', "%%");
    let needs_quotes = argument
        .chars()
        .any(|character| character.is_whitespace() || "\"'\\$`<>|&;*?#()".contains(character));
    if !needs_quotes {
        return argument;
    }

    let escaped: String = argument
        .chars()
        .flat_map(|character| match character {
            '"' | '`' | '$' | '\\' => vec!['\\', character],
            _ => vec![character],
        })
        .collect();
    format!("\"{}\"", escaped.replace('\\', "\\\\"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(opens: Opens, command_line_option: Option<&'static str>) -> DesktopEntry {
        DesktopEntry {
            file_name: "test.desktop",
            name: "Test",
            description: "Test",
            command_line_option,
            opens,
            mime_types: &[],
            shown_in_app_menu: false,
        }
    }

    #[test]
    fn appimage_entries_start_the_appimage_itself() {
        let launcher = Path::new("/home/someone/Apps/Rovarch.AppImage");
        assert_eq!(
            entry(Opens::Link, Some("--roblox")).exec(launcher),
            "/home/someone/Apps/Rovarch.AppImage --roblox %u"
        );
    }

    #[test]
    fn paths_with_spaces_are_quoted() {
        let launcher = Path::new("/home/someone/My Apps/Rovarch.AppImage");
        assert_eq!(
            entry(Opens::File, Some("--studio")).exec(launcher),
            "\"/home/someone/My Apps/Rovarch.AppImage\" --studio %f"
        );
    }
}
