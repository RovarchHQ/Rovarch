use super::account::account_group;
use super::bound_rows::{choice_row, switch_row};
use super::runtime::runtime_section;
use super::session::session_group;
use super::support::support_group;
use super::updates::launcher_version_row;
use crate::i18n::tr;
use crate::state::{AppLanguage, AppState, LaunchMode};
use adw::prelude::*;

pub fn general_section(state: &AppState) -> gtk::Box {
    let group = adw::PreferencesGroup::builder()
        .title(tr("Rovarch"))
        .build();

    let launch_mode_row =
        choice_row::<LaunchMode>(state, "launch-mode", tr("When Rovarch starts").as_str());
    launch_mode_row.set_subtitle(&tr("Settings stay available from the system tray"));

    let run_in_background_row = switch_row(
        state,
        "run-in-background",
        tr("Run in the background").as_str(),
    );
    run_in_background_row.set_subtitle(&tr(
        "Keep Rovarch in the system tray after its window closes",
    ));

    let advanced_row = switch_row(
        state,
        "advanced-settings-visible",
        tr("Advanced settings").as_str(),
    );
    advanced_row.set_subtitle(&tr("Only enable if you know what you're doing."));

    group.add(&launcher_version_row(state));
    let auto_update_row = switch_row(
        state,
        "auto-update",
        tr("Install updates automatically").as_str(),
    );
    auto_update_row.set_subtitle(&tr(
        "Downloads new Rovarch versions and installs them when it closes",
    ));
    group.add(&auto_update_row);
    group.add(&launch_mode_row);
    group.add(&run_in_background_row);
    group.add(&advanced_row);

    let app_language_row =
        choice_row::<AppLanguage>(state, "app-language", tr("App language").as_str());
    app_language_row.set_subtitle(&tr("Changing the language restarts Rovarch."));
    group.add(&app_language_row);

    let discord_rich_presence_row = switch_row(
        state,
        "discord-rich-presence",
        tr("Show what you're playing").as_str(),
    );
    discord_rich_presence_row.set_subtitle(&tr(
        "Show the Roblox game you're in on your Discord profile",
    ));
    let discord_group = adw::PreferencesGroup::builder()
        .title(tr("Discord"))
        .build();
    discord_group.add(&discord_rich_presence_row);

    let section = gtk::Box::new(gtk::Orientation::Vertical, 18);
    section.append(&group);
    section.append(&discord_group);
    section.append(&session_group(state));
    section.append(&super::privacy::privacy_group(state));
    section.append(&runtime_section(state));
    section.append(&account_group(state));
    section.append(&support_group(state));

    section
}
