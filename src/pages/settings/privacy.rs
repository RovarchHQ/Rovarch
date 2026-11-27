use super::bound_rows::switch_row;
use crate::state::AppState;
use adw::prelude::*;

pub fn privacy_group(state: &AppState) -> adw::PreferencesGroup {
    let telemetry_row = switch_row(
        state,
        "block-telemetry",
        crate::i18n::tr("Block Roblox telemetry").as_str(),
    );
    telemetry_row.set_subtitle(
        &crate::i18n::tr(
            "Makes Roblox's telemetry and crash-upload hosts unreachable. Applies the next time Roblox starts",
        ),
    );

    let group = adw::PreferencesGroup::builder()
        .title(crate::i18n::tr("Privacy"))
        .build();
    group.add(&telemetry_row);
    group
}
