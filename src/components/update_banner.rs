use crate::backend;
use crate::i18n::tr_vars;
use crate::state::AppState;
use adw::prelude::*;
use gtk::glib;

pub fn update_banner(state: &AppState) -> adw::Banner {
    let banner = adw::Banner::builder()
        .button_label(crate::i18n::tr("Update"))
        .build();
    banner.connect_button_clicked(glib::clone!(
        #[strong]
        state,
        move |_| backend::install_update(&state)
    ));

    state.follow_properties(
        &["available-update", "busy"],
        glib::clone!(
            #[weak]
            banner,
            move |state| show_update(&banner, state)
        ),
    );

    banner
}

fn show_update(banner: &adw::Banner, state: &AppState) {
    let version = state.available_update();
    banner.set_title(&tr_vars(
        "Rovarch {version} is available",
        &[("version", &version)],
    ));
    banner.set_revealed(!version.is_empty());
    banner.set_sensitive(!state.busy());
}
