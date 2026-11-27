use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallKind {
    AppImage(PathBuf),
    Other,
}

impl InstallKind {
    pub fn detect() -> Self {
        Self::from_environment(std::env::var_os("APPIMAGE"))
    }

    fn from_environment(appimage: Option<OsString>) -> Self {
        match appimage.filter(|path| !path.is_empty()).map(PathBuf::from) {
            Some(path) if path.is_file() => Self::AppImage(path),
            _ => Self::Other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::InstallKind;

    #[test]
    fn tells_the_ways_rovarch_is_installed_apart() {
        let existing = std::env::current_exe().unwrap();
        assert_eq!(
            InstallKind::from_environment(Some(existing.clone().into_os_string())),
            InstallKind::AppImage(existing.clone())
        );
        assert_eq!(InstallKind::from_environment(None), InstallKind::Other);
        assert_eq!(
            InstallKind::from_environment(Some("/nowhere/Rovarch.AppImage".into())),
            InstallKind::Other
        );
        assert_eq!(
            InstallKind::from_environment(Some("".into())),
            InstallKind::Other
        );
    }
}
