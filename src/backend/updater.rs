mod appimage;
mod install_kind;
mod release;
pub mod version;

use super::background::{null_progress, run_quietly, run_task};
use crate::i18n::{tr, tr_vars};
use crate::state::AppState;
use gtk::glib;
use install_kind::InstallKind;
use release::AppImageUpdate;
use std::cell::{Cell, RefCell};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const VARIABLES_THE_APPIMAGE_RUNTIME_SETS: [&str; 4] = ["APPIMAGE", "APPDIR", "ARGV0", "OWD"];
const RECHECK_AFTER: u32 = 6 * 60 * 60;

thread_local! {
    static INSTALL_KIND: InstallKind = InstallKind::detect();
    static AVAILABLE_UPDATE: RefCell<Option<AppImageUpdate>> = const { RefCell::new(None) };
    static CHECKING: Cell<bool> = const { Cell::new(false) };
    static STAGED_UPDATE: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
    static DOWNLOADING_UPDATE: Cell<bool> = const { Cell::new(false) };
}

pub fn initialize(state: &AppState) {
    match INSTALL_KIND.with(Clone::clone) {
        InstallKind::AppImage(_) => {
            state.follow_properties(
                &["available-update", "auto-update", "busy", "running"],
                glib::clone!(
                    #[weak]
                    state,
                    move |_| stage_update_in_background(&state)
                ),
            );
            let timer_state = state.clone();
            glib::timeout_add_seconds_local(RECHECK_AFTER, move || {
                check(&timer_state);
                glib::ControlFlow::Continue
            });
            check(state);
        }
        InstallKind::Other => {
            state.set_update_status(tr("Updates come from wherever you installed Rovarch"));
        }
    }
}

fn stage_update_in_background(state: &AppState) {
    if !state.auto_update()
        || state.busy()
        || state.running()
        || DOWNLOADING_UPDATE.get()
        || STAGED_UPDATE.with_borrow(Option::is_some)
    {
        return;
    }
    let Some(update) = AVAILABLE_UPDATE.with_borrow(Clone::clone) else {
        return;
    };
    let Some(running_appimage) = running_appimage() else {
        return;
    };

    DOWNLOADING_UPDATE.set(true);
    let state = state.clone();
    run_quietly(
        move || appimage::stage(&update, &running_appimage, &null_progress()),
        move |staged| {
            DOWNLOADING_UPDATE.set(false);
            match staged {
                Ok(staged) => {
                    STAGED_UPDATE.replace(Some(staged));
                    state.set_update_status(tr("Restart Rovarch to finish the update"));
                }
                Err(error) => {
                    eprintln!("Couldn't download the update in the background: {error:#}")
                }
            }
        },
    );
}

pub fn finish_pending_update() {
    let Some(staged) = STAGED_UPDATE.with_borrow_mut(Option::take) else {
        return;
    };
    let Some(running_appimage) = running_appimage() else {
        return;
    };
    if let Err(error) = std::fs::rename(&staged, &running_appimage) {
        eprintln!("Couldn't install the downloaded update: {error}");
    }
}

pub fn can_update_itself() -> bool {
    matches!(INSTALL_KIND.with(Clone::clone), InstallKind::AppImage(_))
}

pub fn running_appimage() -> Option<PathBuf> {
    match INSTALL_KIND.with(Clone::clone) {
        InstallKind::AppImage(appimage) => Some(appimage),
        InstallKind::Other => None,
    }
}

pub fn check(state: &AppState) {
    if !can_update_itself() || CHECKING.replace(true) {
        return;
    }
    state.set_update_status(tr("Checking for updates…"));

    let state = state.clone();
    run_quietly(
        || release::newest_after(CURRENT_VERSION),
        move |newest| {
            match newest {
                Ok(Some(update)) => {
                    state.set_update_status(tr_vars(
                        "Version {version} is available",
                        &[("version", &update.version)],
                    ));
                    state.set_available_update(update.version.clone());
                    AVAILABLE_UPDATE.replace(Some(update));
                }
                Ok(None) => {
                    state.set_update_status(tr("Rovarch is up to date"));
                    state.set_available_update("");
                    AVAILABLE_UPDATE.replace(None);
                }
                Err(error) => {
                    eprintln!("{error:#}");
                    state.set_update_status(tr("Couldn't check for updates"));
                }
            }
            CHECKING.set(false);
        },
    );
}

pub fn install(state: &AppState) {
    if state.busy() || state.running() {
        state.report_error("Close Roblox and wait for Rovarch to finish before updating.");
        return;
    }
    let InstallKind::AppImage(running_appimage) = INSTALL_KIND.with(Clone::clone) else {
        return;
    };
    let Some(update) = AVAILABLE_UPDATE.with_borrow(Clone::clone) else {
        return;
    };

    let appimage_to_restart = running_appimage.clone();
    run_task(
        state,
        move |progress| appimage::replace(&update, &running_appimage, progress),
        move |state, ()| restart(state, &appimage_to_restart),
    );
}

pub fn restart_now() {
    let running_appimage = running_appimage();
    let mut command = match &running_appimage {
        Some(appimage) => {
            let mut command = Command::new(appimage);
            for variable in VARIABLES_THE_APPIMAGE_RUNTIME_SETS {
                command.env_remove(variable);
            }
            command
        }
        None => Command::new("/proc/self/exe"),
    };
    let error = command.exec();
    eprintln!("Couldn't restart Rovarch: {error}");
}

fn restart(state: &AppState, appimage: &Path) {
    state.save_pending_changes();
    let mut command = Command::new(appimage);
    for variable in VARIABLES_THE_APPIMAGE_RUNTIME_SETS {
        command.env_remove(variable);
    }
    let error = command.exec();
    state.report_error(&format!(
        "Rovarch updated, but couldn't restart itself ({error}). Open it again to use the new version."
    ));
}
