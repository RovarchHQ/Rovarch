use crate::i18n::{tr, tr_vars};
use crate::state::{AppState, Client};

pub const PROPERTIES_THAT_DECIDE_STATUS: [&str; 13] = [
    "busy",
    "has-busy-client",
    "busy-client",
    "running",
    "running-client",
    "selected-client",
    "roblox-graphics-started",
    "task-description",
    "task-progress",
    "roblox-installed",
    "roblox-version",
    "studio-installed",
    "studio-version",
];

const ROVARCH_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn status_text(state: &AppState, client: Client) -> String {
    let is_opening = state.is_running(client) && !state.roblox_graphics_started();

    if state.is_busy_for(client) {
        let desc = state.task_description();
        let desc = if desc.is_empty() {
            tr("Starting…")
        } else {
            desc
        };
        let percent = (state.task_progress() * 100.0).round();
        if percent > 0.0 {
            format!("{desc} ({percent}%)")
        } else {
            format!("{desc}…")
        }
    } else if is_opening {
        tr_vars("Opening {client}…", &[("client", client.name())])
    } else if state.is_installed(client) {
        format!(
            "{} {} • Rovarch {}",
            client.name(),
            state.version_of(client),
            ROVARCH_VERSION
        )
    } else {
        tr_vars("{client} isn't installed", &[("client", client.name())])
    }
}
