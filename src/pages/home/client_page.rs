use super::logs_button::logs_button;
use super::play_button::play_button;
use super::status_label::status_label;
use crate::components::hero_glow::hero_glow;
use crate::components::icon::client_logo;
use crate::state::{AppState, Client};
use gtk::prelude::*;

pub fn client_page(state: &AppState, client: Client) -> gtk::Widget {
    let emblem = client_logo(client, 84);

    let title = gtk::Label::builder()
        .label(client.home_title())
        .css_classes(["hero-title"])
        .build();

    let subtitle = status_label(state, client);

    let play_area = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(10)
        .halign(gtk::Align::Center)
        .margin_top(4)
        .build();
    play_area.append(&play_button(state, client));
    play_area.append(&logs_button(state));

    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(10)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .vexpand(true)
        .margin_top(16)
        .margin_bottom(24)
        .build();
    content.append(&emblem);
    content.append(&title);
    content.append(&subtitle);
    content.append(&play_area);

    let overlay = gtk::Overlay::builder().child(&hero_glow(&emblem)).build();
    overlay.add_overlay(&content);
    overlay.set_measure_overlay(&content, true);

    overlay.upcast()
}
