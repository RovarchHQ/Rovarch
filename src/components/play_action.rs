use crate::backend::{self, StudioTarget};
use crate::i18n::tr;
use crate::state::{AppState, Client};

pub const PROPERTIES_THAT_DECIDE_PLAY_ACTION: [&str; 11] = [
    "running",
    "running-client",
    "selected-client",
    "stopping",
    "roblox-graphics-started",
    "busy",
    "can-play",
    "can-open-studio",
    "roblox-installed",
    "studio-installed",
    "rosemary-installed",
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PlayAction {
    Install(Client),
    Play(Client),
    Opening,
    Stop,
    Stopping,
}

impl PlayAction {
    pub fn for_client(state: &AppState, client: Client) -> Self {
        let is_running = state.is_running(client);

        if is_running && state.stopping() {
            Self::Stopping
        } else if is_running && !state.roblox_graphics_started() {
            Self::Opening
        } else if is_running {
            Self::Stop
        } else if !state.is_installed(client) {
            Self::Install(client)
        } else {
            Self::Play(client)
        }
    }

    pub fn is_available(self, state: &AppState) -> bool {
        match self {
            Self::Install(_) => !state.busy(),
            Self::Play(client) => state.can_start(client) && !state.busy() && !state.running(),
            Self::Stop => true,
            Self::Opening | Self::Stopping => false,
        }
    }

    pub fn explanation_when_unavailable(self, state: &AppState) -> Option<String> {
        match self {
            Self::Play(_) if state.running() => match state.running_client() {
                Client::Player => Some(tr("Rovarch Play is already running.")),
                Client::Studio => Some(tr("Rovarch Studio is already running.")),
            },
            Self::Play(client) if !state.can_start(client) => Some(tr(
                "The selected runner isn't installed. Choose another one in Settings.",
            )),
            _ => None,
        }
    }

    pub fn label(self) -> String {
        match self {
            Self::Install(Client::Player) => tr("Install Rovarch Play"),
            Self::Install(Client::Studio) => tr("Install Rovarch Studio"),
            Self::Play(Client::Player) => tr("Play"),
            Self::Play(Client::Studio) => tr("Open"),
            Self::Opening => tr("Opening…"),
            Self::Stop => tr("Stop"),
            Self::Stopping => tr("Stopping…"),
        }
    }

    pub fn perform(self, state: &AppState) {
        match self {
            Self::Install(client) => backend::install(state, client),
            Self::Play(Client::Player) => backend::play(state),
            Self::Play(Client::Studio) => backend::open_studio(state, StudioTarget::StartPage),
            Self::Stop => backend::stop(state),
            Self::Opening | Self::Stopping => {}
        }
    }
}
