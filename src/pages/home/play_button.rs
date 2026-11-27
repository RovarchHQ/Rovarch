use crate::components::play_action::{PROPERTIES_THAT_DECIDE_PLAY_ACTION, PlayAction};
use crate::state::{AppState, Client};
use adw::prelude::*;
use gtk::glib;

const ACTION_STYLE_CLASSES: [&str; 2] = ["suggested-action", "destructive-action"];

fn style_class(action: PlayAction) -> Option<&'static str> {
    match action {
        PlayAction::Install(_) | PlayAction::Play(_) => Some("suggested-action"),
        PlayAction::Stop => Some("destructive-action"),
        PlayAction::Opening | PlayAction::Stopping => None,
    }
}

pub fn play_button(state: &AppState, client: Client) -> gtk::Button {
    let button = gtk::Button::builder()
        .css_classes(["hero-play"])
        .halign(gtk::Align::Center)
        .build();

    button.connect_clicked(glib::clone!(
        #[strong]
        state,
        move |_| PlayAction::for_client(&state, client).perform(&state)
    ));

    state.follow_properties(
        &PROPERTIES_THAT_DECIDE_PLAY_ACTION,
        glib::clone!(
            #[weak]
            button,
            move |state| show_action(&button, state, client)
        ),
    );

    button
}

fn show_action(button: &gtk::Button, state: &AppState, client: Client) {
    let action = PlayAction::for_client(state, client);

    button.set_label(&action.label());

    for style_class in ACTION_STYLE_CLASSES {
        button.remove_css_class(style_class);
    }
    if let Some(style_class) = style_class(action) {
        button.add_css_class(style_class);
    }

    button.set_sensitive(action.is_available(state));
    button.set_tooltip_text(action.explanation_when_unavailable(state).as_deref());
}
