use super::background::{Progress, run_quietly};
use super::download::{
    download_file, remove_directory_if_present, replace_directory, stop_others_from_writing,
};
use super::req_agent::api_agent;
use super::{fast_flags, paths};
use crate::i18n::tr_vars;
use crate::state::{AppState, Client};
use anyhow::{Context, ensure};
use gtk::gio;
use gtk::gio::prelude::*;
use gtk::glib;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::path::PathBuf;

const LATEST_RELEASE_BASE_URL: &str = "https://clientsettingscdn.roblox.com/v2/client-version";
const DOWNLOAD_BASE_URL: &str = "https://setup.rbxcdn.com/mac";
const INSTALLED_RELEASE_FILE: &str = "release.toml";

const FIRST_UPDATE_CHECK: u32 = 45;
const UPDATE_WATCH_EVERY: u32 = 6 * 60 * 60;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RobloxRelease {
    pub version: String,
    pub client_version_upload: String,
}

struct ClientFiles {
    release_channel: &'static str,
    archive_suffix: &'static str,
    app_name: &'static str,
    executable_inside_app: &'static str,
    directory: PathBuf,
}

fn files_of(client: Client) -> ClientFiles {
    match client {
        Client::Player => ClientFiles {
            release_channel: "MacPlayer",
            archive_suffix: "RobloxPlayer.zip",
            app_name: "RobloxPlayer.app",
            executable_inside_app: "Contents/MacOS/RobloxPlayer",
            directory: paths::roblox_directory(),
        },
        Client::Studio => ClientFiles {
            release_channel: "MacStudio",
            archive_suffix: "RobloxStudioApp.zip",
            app_name: "RobloxStudio.app",
            executable_inside_app: "Contents/MacOS/RobloxStudio",
            directory: paths::studio_directory(),
        },
    }
}

pub fn app_path(client: Client) -> PathBuf {
    let files = files_of(client);
    files.directory.join(files.app_name)
}

pub fn start_update_watch(state: &AppState) {
    check_for_client_updates_after(state, FIRST_UPDATE_CHECK);
}

fn check_for_client_updates_after(state: &AppState, seconds: u32) {
    let state = state.clone();
    glib::timeout_add_seconds_local(seconds, move || {
        notify_of_new_releases(&state);
        check_for_client_updates_after(&state, UPDATE_WATCH_EVERY);
        glib::ControlFlow::Break
    });
}

fn notify_of_new_releases(state: &AppState) {
    for client in Client::ALL {
        if state.is_running(client) || state.busy() || installed_release(client).is_none() {
            continue;
        }
        run_quietly(
            move || {
                let latest = latest_release(client)?;
                let installed = installed_release(client)
                    .map(|release| release.version)
                    .unwrap_or_default();
                anyhow::Ok((latest.version != installed).then_some(latest.version))
            },
            move |update| match update {
                Ok(Some(version)) => notify_of_update(client, &version),
                Ok(None) => {}
                Err(error) => eprintln!("Couldn't check {client:?} for updates: {error:#}"),
            },
        );
    }
}

fn notify_of_update(client: Client, version: &str) {
    let Some(application) = gio::Application::default() else {
        return;
    };
    let notification = gio::Notification::new(client.name());
    notification.set_body(Some(&tr_vars(
        "Version {version} is available. It installs the next time you play.",
        &[("version", version)],
    )));
    notification.set_icon(&gio::ThemedIcon::new(crate::APP_ID));
    application.send_notification(Some("client-update"), &notification);
}

pub fn installed_release(client: Client) -> Option<RobloxRelease> {
    let files = files_of(client);
    if !app_path(client).join(files.executable_inside_app).is_file() {
        return None;
    }
    let release = fs::read_to_string(files.directory.join(INSTALLED_RELEASE_FILE)).ok()?;
    toml::from_str(&release).ok()
}

pub fn latest_release(client: Client) -> anyhow::Result<RobloxRelease> {
    let url = format!(
        "{LATEST_RELEASE_BASE_URL}/{}",
        files_of(client).release_channel
    );
    api_agent()
        .get(&url)
        .call()
        .and_then(|mut response| response.body_mut().read_json())
        .with_context(|| format!("Couldn't check for the latest {} version", client.name()))
}

pub fn install(client: Client, release: &RobloxRelease, progress: &Progress) -> anyhow::Result<()> {
    let files = files_of(client);
    let name = client.name();
    let archive_name = format!("{}-{}", release.client_version_upload, files.archive_suffix);
    let archive = paths::downloads_directory().join(&archive_name);
    let is_update = installed_release(client).is_some();
    if is_update {
        progress.step(format!("Downloading the {name} {} update", release.version));
    } else {
        progress.step(format!("Downloading {name} {}", release.version));
    }
    download_file(&download_url(&archive_name), &archive, progress)?;

    if is_update {
        progress.step(format!("Updating {name} to {}", release.version));
    } else {
        progress.step(format!("Installing {name} {}", release.version));
    }
    let unpacking = files.directory.join(".unpacking");
    remove_directory_if_present(&unpacking)?;
    zip::ZipArchive::new(File::open(&archive)?)
        .and_then(|mut zip| zip.extract(&unpacking))
        .with_context(|| format!("Couldn't unpack the {name} download"))?;
    stop_others_from_writing(&unpacking)?;

    let unpacked_app = unpacking.join(files.app_name);
    ensure!(
        unpacked_app.join(files.executable_inside_app).is_file(),
        "The {name} download has no {}.",
        files.app_name
    );
    replace_directory(&unpacked_app, &app_path(client))?;
    remove_directory_if_present(&unpacking)?;
    if client == Client::Player {
        fast_flags::reapply_remembered()?;
    }
    fs::write(
        files.directory.join(INSTALLED_RELEASE_FILE),
        toml::to_string(release)?,
    )?;
    let _ = fs::remove_file(&archive);

    Ok(())
}

pub fn install_or_update(
    client: Client,
    installed: Option<&RobloxRelease>,
    progress: &Progress,
) -> anyhow::Result<()> {
    progress.step(format!("Checking for {} updates", client.name()));
    match latest_release(client) {
        Ok(latest) if Some(&latest) != installed => install(client, &latest, progress),
        Ok(_) => Ok(()),
        Err(error) if installed.is_some() => {
            eprintln!(
                "Starting the installed {} without updating: {error:#}",
                client.name()
            );
            Ok(())
        }
        Err(error) => Err(error),
    }
}

fn download_url(archive_name: &str) -> String {
    match std::env::consts::ARCH {
        "aarch64" => format!("{DOWNLOAD_BASE_URL}/arm64/{archive_name}"),
        _ => format!("{DOWNLOAD_BASE_URL}/{archive_name}"),
    }
}
