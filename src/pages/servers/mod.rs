mod history_group;
mod join_availability;
mod join_by_link_row;
mod place_link;
mod region_row;
mod time_since;

use crate::backend;
use crate::i18n::tr;
use crate::state::AppState;
use crate::utils::{scrollable, unfocus_on_outside_click};
use adw::prelude::*;
use gtk::glib;
use history_group::history_groups;
use join_availability::JoinAvailability;
use join_by_link_row::join_by_link_row;
use region_row::preferred_region_row;

pub fn servers_page(state: &AppState) -> gtk::Widget {
    let join_availability = JoinAvailability::follow(state);

    let heading = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .build();
    let labels = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(2)
        .hexpand(true)
        .build();
    labels.append(
        &gtk::Label::builder()
            .label(tr("Servers & Network").as_str())
            .halign(gtk::Align::Start)
            .css_classes(["title-2"])
            .build(),
    );
    labels.append(
        &gtk::Label::builder()
            .label(tr("Regional Roblox datacenters and your session history"))
            .halign(gtk::Align::Start)
            .css_classes(["dimmed"])
            .wrap(true)
            .build(),
    );
    heading.append(&labels);

    let test_latency = gtk::Button::builder()
        .label(tr("Test Latency"))
        .valign(gtk::Align::Center)
        .build();
    test_latency.connect_clicked(glib::clone!(
        #[strong]
        state,
        move |button| {
            button.set_sensitive(false);
            button.set_label(&tr("Testing…"));
            backend::refresh_regions(
                &state,
                glib::clone!(
                    #[weak]
                    button,
                    move |result| {
                        button.set_sensitive(true);
                        match result {
                            Ok(ms) => button.set_label(&format!("{ms} ms")),
                            Err(_) => button.set_label(&tr("Test Latency")),
                        }
                    }
                ),
            );
        }
    ));
    heading.append(&test_latency);

    let joining_group = adw::PreferencesGroup::builder()
        .title(tr("Joining"))
        .description(tr(
            "With a preferred region, games started from this page join a server there, \
             picked from RoValra's server list (apis.rovalra.com). RoValra sees the game and \
             the region, not your account.",
        ))
        .build();
    joining_group.add(&preferred_region_row(state));
    joining_group.add(&join_by_link_row(state, &join_availability));

    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .margin_top(12)
        .margin_bottom(12)
        .build();
    content.append(&heading);
    content.append(&region_cards_grid(state));
    content.append(&joining_group);
    content.append(&history_groups(state, &join_availability));

    let page = scrollable(&content);
    unfocus_on_outside_click(&page);
    load_regions_when_first_shown(&page, state);

    page.upcast()
}

fn region_cards_grid(state: &AppState) -> gtk::FlowBox {
    let grid = gtk::FlowBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .homogeneous(true)
        .min_children_per_line(2)
        .max_children_per_line(3)
        .column_spacing(12)
        .row_spacing(12)
        .build();

    state.follow_properties(
        &["regions", "preferred-region"],
        glib::clone!(
            #[weak]
            grid,
            move |state| populate_region_grid(&grid, state)
        ),
    );

    grid
}

fn populate_region_grid(grid: &gtk::FlowBox, state: &AppState) {
    while let Some(child) = grid.first_child() {
        grid.remove(&child);
    }

    let regions = state.regions();
    let preferred = state.preferred_region();
    grid.set_visible(!regions.0.is_empty());

    for region in regions.0.iter().take(6) {
        let key = region.key();
        let is_selected = key == preferred;

        let card = gtk::Button::builder()
            .css_classes(if is_selected {
                vec!["region-card", "selected"]
            } else {
                vec!["region-card"]
            })
            .valign(gtk::Align::Fill)
            .build();

        let row = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(12)
            .valign(gtk::Align::Center)
            .build();
        let labels = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(3)
            .hexpand(true)
            .valign(gtk::Align::Center)
            .build();
        labels.append(
            &gtk::Label::builder()
                .label(&region.country_name)
                .halign(gtk::Align::Start)
                .ellipsize(gtk::pango::EllipsizeMode::End)
                .build(),
        );
        labels.append(
            &gtk::Label::builder()
                .label(region.city.as_deref().unwrap_or(&region.country_code))
                .halign(gtk::Align::Start)
                .css_classes(["dimmed", "caption"])
                .ellipsize(gtk::pango::EllipsizeMode::End)
                .build(),
        );

        let badge = gtk::Label::builder()
            .label(&region.country_code)
            .css_classes(["region-code-badge"])
            .valign(gtk::Align::Center)
            .build();

        row.append(&labels);
        row.append(&badge);
        card.set_child(Some(&row));

        card.connect_clicked(glib::clone!(
            #[weak]
            state,
            move |_| {
                if state.preferred_region() == key {
                    state.set_preferred_region(String::new());
                } else {
                    state.set_preferred_region(key.clone());
                }
            }
        ));

        grid.insert(&card, -1);
    }
}

fn load_regions_when_first_shown(page: &gtk::ScrolledWindow, state: &AppState) {
    let handler = std::rc::Rc::new(std::cell::Cell::new(None));
    let first_map = page.connect_map(glib::clone!(
        #[strong]
        state,
        #[strong]
        handler,
        move |page| {
            backend::load_regions(&state);
            if let Some(handler) = handler.take() {
                page.disconnect(handler);
            }
        }
    ));
    handler.set(Some(first_map));
}
