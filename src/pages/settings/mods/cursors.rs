use crate::components::file_row::{Pick, file_row};
use crate::i18n::tr;
use crate::pages::settings::bound_rows::{choice_row, show_when};
use crate::state::{AppState, CursorStyle};
use adw::prelude::*;

pub fn cursors_group(state: &AppState) -> adw::PreferencesGroup {
    let style_row = choice_row::<CursorStyle>(state, "cursor-style", "Cursor style");
    style_row.set_subtitle(&tr("Replaces the mouse cursors inside the game"));

    let custom_row = file_row(
        state,
        "custom-cursor",
        &tr("Custom cursor"),
        &[Pick::File(&["png"]), Pick::Folder],
    );
    show_when(&custom_row, state, "cursor-style", |style: CursorStyle| {
        style == CursorStyle::Custom
    });

    let group = adw::PreferencesGroup::builder()
        .title(tr("Cursors"))
        .build();
    group.add(&style_row);
    group.add(&custom_row);
    group
}
