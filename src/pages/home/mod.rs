mod client_page;
mod logs_button;
mod play_button;
mod status_label;

use crate::state::{AppState, Client};
use adw::prelude::*;
use client_page::client_page;
use gtk::glib;

pub fn home_page(state: &AppState) -> gtk::Widget {
    let clients = adw::ViewStack::builder().vexpand(true).build();
    for client in Client::ALL {
        clients.add_named(&client_page(state, client), Some(client.page_name()));
    }

    state.follow_properties(
        &["selected-client"],
        glib::clone!(
            #[weak]
            clients,
            move |state| {
                clients.set_visible_child_name(state.selected_client().page_name());
            }
        ),
    );

    clients.upcast()
}
