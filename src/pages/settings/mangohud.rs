use super::bound_rows::switch_row;
use crate::i18n::tr;
use crate::state::AppState;
use adw::prelude::*;

pub fn mangohud_row(state: &AppState) -> adw::SwitchRow {
    let row = switch_row(state, "mangohud", "MangoHud");
    row.set_subtitle(&tr(
        "Show an overlay with the frame rate, frame times and GPU usage",
    ));

    row
}
