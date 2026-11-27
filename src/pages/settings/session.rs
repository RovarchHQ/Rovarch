use super::bound_rows::{stepper_row, switch_row};
use crate::state::AppState;
use adw::prelude::*;

pub fn session_group(state: &AppState) -> adw::PreferencesGroup {
    let hide_row = switch_row(
        state,
        "hide-launcher-while-playing",
        crate::i18n::tr("Hide Rovarch while playing").as_str(),
    );
    hide_row.set_subtitle(&crate::i18n::tr(
        "Closes the window once Roblox opens. Rovarch quits with the game",
    ));

    let reopen_row = switch_row(
        state,
        "open-after-exit",
        crate::i18n::tr("Open Rovarch after Roblox exits").as_str(),
    );
    let playtime_row = switch_row(
        state,
        "show-playtime",
        crate::i18n::tr("Show playtime").as_str(),
    );
    playtime_row.set_subtitle(&crate::i18n::tr(
        "Show your total Roblox playtime on the home page",
    ));

    let location_row = switch_row(
        state,
        "notify-server-location",
        crate::i18n::tr("Notify of server location").as_str(),
    );
    location_row.set_subtitle(&crate::i18n::tr(
        "Looks up where your server is through RoValra and shows a notification. \
         Only the server's address is sent",
    ));

    let group = adw::PreferencesGroup::builder()
        .title(crate::i18n::tr("While playing"))
        .build();
    group.add(&hide_row);
    group.add(&reopen_row);
    group.add(&playtime_row);
    group.add(&location_row);
    group.add(&cpu_core_limit_row(state));
    group.add(&video_memory_row(state));
    group
}

fn cpu_core_limit_row(state: &AppState) -> adw::ActionRow {
    let cores = std::thread::available_parallelism().map_or(1, |cores| cores.get()) as u32;
    stepper_row(
        state,
        "cpu-core-limit",
        "Limit CPU cores",
        "Experimental. 0 lets Roblox use every core",
        0,
        cores,
        1,
    )
}

const MAX_VIDEO_MEMORY_MB: u32 = 65536;
const VIDEO_MEMORY_STEP_MB: u32 = 256;

fn video_memory_row(state: &AppState) -> adw::ActionRow {
    stepper_row(
        state,
        "video-memory-mb",
        "Video memory (MB)",
        "Tells Roblox how much video memory you have. 0 detects it. Raise it if textures look blurry on an integrated GPU",
        0,
        MAX_VIDEO_MEMORY_MB,
        VIDEO_MEMORY_STEP_MB,
    )
}
