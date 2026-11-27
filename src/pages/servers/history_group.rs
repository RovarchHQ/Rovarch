use super::join_availability::JoinAvailability;
use super::time_since::time_since;
use crate::backend;
use crate::i18n::{tr, tr_vars};
use crate::state::{AppState, JoinedServer, PlayedGame};
use adw::prelude::*;
use gtk::glib;
use std::cell::RefCell;
use std::rc::Rc;

fn unknown_location() -> String {
    tr("Unknown location")
}

type ShownRows = Rc<RefCell<Vec<gtk::Widget>>>;

pub fn history_groups(
    state: &AppState,
    join_availability: &JoinAvailability,
) -> adw::PreferencesGroup {
    let history_group = adw::PreferencesGroup::builder()
        .title(tr("Server history"))
        .description(tr(
            "Servers you joined with Rovarch. Rejoining asks Roblox for the same server, \
             and Roblox picks a new one if it has closed.",
        ))
        .build();

    let forget_button = gtk::Button::builder()
        .label(tr("Clear history"))
        .valign(gtk::Align::Center)
        .build();
    forget_button.connect_clicked(glib::clone!(
        #[weak]
        state,
        move |_| backend::forget_server_history(&state)
    ));
    history_group.set_header_suffix(Some(&forget_button));

    let shown_rows = ShownRows::default();
    show_history(
        &history_group,
        &forget_button,
        &shown_rows,
        state,
        join_availability,
    );

    state.connect_server_history_notify(glib::clone!(
        #[weak]
        history_group,
        #[weak]
        forget_button,
        #[strong]
        shown_rows,
        #[strong]
        join_availability,
        move |state| show_history(
            &history_group,
            &forget_button,
            &shown_rows,
            state,
            &join_availability
        )
    ));

    history_group
}

fn show_history(
    history_group: &adw::PreferencesGroup,
    forget_button: &gtk::Button,
    shown_rows: &ShownRows,
    state: &AppState,
    join_availability: &JoinAvailability,
) {
    for row in shown_rows.borrow_mut().drain(..) {
        history_group.remove(&row);
    }

    let history = state.server_history();
    let rows: Vec<gtk::Widget> = if history.0.is_empty() {
        vec![nothing_played_row().upcast()]
    } else {
        history
            .favorites_first()
            .into_iter()
            .map(|game| game_row(state, game, join_availability).upcast())
            .collect()
    };

    for row in &rows {
        history_group.add(row);
    }
    forget_button.set_visible(!history.0.is_empty());
    shown_rows.replace(rows);
}

fn nothing_played_row() -> adw::ActionRow {
    adw::ActionRow::builder()
        .title(tr("No servers yet"))
        .subtitle(tr("Games you join will show up here."))
        .build()
}

fn game_row(
    state: &AppState,
    game: &PlayedGame,
    join_availability: &JoinAvailability,
) -> adw::ExpanderRow {
    let title = if game.name.is_empty() {
        tr_vars(
            "Place {place_id}",
            &[("place_id", &game.place_id.to_string())],
        )
    } else {
        game.name.clone()
    };
    let row = adw::ExpanderRow::builder()
        .title(title)
        .subtitle(game.last_played_at().map(time_since).unwrap_or_default())
        .use_markup(false)
        .build();

    let place_id = game.place_id;
    if let Some(url) = &game.icon_url {
        let thumbnail = gtk::Image::builder().pixel_size(36).build();
        row.add_prefix(&thumbnail);
        let url = url.clone();
        backend::load_game_icon(
            place_id,
            &url,
            glib::clone!(
                #[weak]
                thumbnail,
                move |path| {
                    if let Some(path) = path
                        && let Ok(texture) =
                            gtk::gdk::Texture::from_file(&gtk::gio::File::for_path(&path))
                    {
                        thumbnail.set_paintable(Some(&texture));
                    }
                }
            ),
        );
    }

    let favorite_button = gtk::Button::builder()
        .icon_name(if game.favorite {
            "starred-symbolic"
        } else {
            "non-starred-symbolic"
        })
        .tooltip_text(tr("Favorite").as_str())
        .valign(gtk::Align::Center)
        .build();
    favorite_button.add_css_class("flat");
    favorite_button.connect_clicked(glib::clone!(
        #[weak]
        state,
        move |_| backend::toggle_favorite_server(&state, place_id)
    ));
    row.add_suffix(&favorite_button);
    let play_button = suffix_button(tr("Play"));
    play_button.add_css_class("suggested-action");
    play_button.connect_clicked(glib::clone!(
        #[weak]
        state,
        move |_| backend::join_game(&state, place_id)
    ));
    join_availability.applies_to(&play_button);
    row.add_suffix(&play_button);

    for server in &game.servers {
        row.add_row(&server_row(state, place_id, server, join_availability));
    }

    row
}

fn server_row(
    state: &AppState,
    place_id: u64,
    server: &JoinedServer,
    join_availability: &JoinAvailability,
) -> adw::ActionRow {
    let title = server
        .location
        .as_deref()
        .or(server.address.as_deref())
        .map_or_else(unknown_location, str::to_owned);
    let row = adw::ActionRow::builder()
        .title(title)
        .subtitle(time_since(server.joined_at))
        .use_markup(false)
        .build();

    let job_id = server.job_id.clone();
    let rejoin_button = suffix_button(tr("Rejoin"));
    rejoin_button.connect_clicked(glib::clone!(
        #[weak]
        state,
        move |_| backend::rejoin_server(&state, place_id, &job_id)
    ));
    join_availability.applies_to(&rejoin_button);
    row.add_suffix(&rejoin_button);

    row
}

fn suffix_button(label: String) -> gtk::Button {
    gtk::Button::builder()
        .label(label)
        .valign(gtk::Align::Center)
        .build()
}
