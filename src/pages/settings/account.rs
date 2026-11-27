use crate::backend;
use crate::i18n::tr;
use crate::state::AppState;
use adw::prelude::*;
use gtk::glib;

const CANCEL_RESPONSE: &str = "cancel";
const SIGN_OUT_RESPONSE: &str = "sign-out";
const PROPERTIES_THAT_DECIDE_SIGN_OUT: [&str; 2] = ["signed-in", "running"];

pub fn account_group(state: &AppState) -> adw::PreferencesGroup {
    let sign_out_button = gtk::Button::builder()
        .label(tr("Sign out"))
        .css_classes(["destructive-action"])
        .valign(gtk::Align::Center)
        .build();
    sign_out_button.connect_clicked(glib::clone!(
        #[strong]
        state,
        move |button| confirm_sign_out(button, &state)
    ));

    let row = adw::ActionRow::builder()
        .title(tr("Roblox account"))
        .build();
    row.add_suffix(&sign_out_button);
    state.follow_properties(
        &PROPERTIES_THAT_DECIDE_SIGN_OUT,
        glib::clone!(
            #[weak]
            row,
            #[weak]
            sign_out_button,
            move |state| show_account(&row, &sign_out_button, state)
        ),
    );

    let group = adw::PreferencesGroup::builder()
        .title(tr("Account"))
        .build();
    group.add(&row);
    group
}

fn show_account(row: &adw::ActionRow, sign_out_button: &gtk::Button, state: &AppState) {
    row.set_subtitle(&if state.signed_in() {
        tr("Signed in. The session is stored only on this computer")
    } else {
        tr("Not signed in")
    });
    sign_out_button.set_sensitive(state.signed_in() && !state.running());
}

fn confirm_sign_out(button: &gtk::Button, state: &AppState) {
    let dialog = adw::AlertDialog::builder()
        .heading(tr("Sign out of Roblox?"))
        .body(tr(
            "Rovarch deletes the saved session. You'll sign in again the next time you play.",
        ))
        .build();
    dialog.add_response(CANCEL_RESPONSE, &tr("Cancel"));
    dialog.add_response(SIGN_OUT_RESPONSE, &tr("Sign out"));
    dialog.set_response_appearance(SIGN_OUT_RESPONSE, adw::ResponseAppearance::Destructive);
    dialog.set_close_response(CANCEL_RESPONSE);

    dialog.connect_response(
        Some(SIGN_OUT_RESPONSE),
        glib::clone!(
            #[strong]
            state,
            move |_, _| backend::sign_out(&state)
        ),
    );
    dialog.present(Some(button));
}
