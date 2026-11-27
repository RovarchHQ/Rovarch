use crate::components::logs_dialog::show_logs;
use crate::i18n::tr;
use crate::state::AppState;
use adw::prelude::*;
use gtk::glib;

pub fn logs_button(state: &AppState) -> gtk::Button {
    let button = gtk::Button::builder()
        .label(tr("Show Logs"))
        .css_classes(["flat"])
        .halign(gtk::Align::Center)
        .margin_top(4)
        .build();

    button.connect_clicked(glib::clone!(
        #[strong]
        state,
        move |button| show_logs(&state, button)
    ));

    button
}
