mod about;
pub use about::show_about;
mod account;
mod advanced;
mod bound_rows;
mod custom_flags;
mod environment_variables;
mod fast_flag_allowlist;
mod fast_flags;
mod general;
mod integration;
mod launch_wrapper;
mod mangohud;
mod mods;
mod privacy;
mod rosemary_options;
mod rosemary_runtime;
mod runtime;
mod session;
mod support;
mod updates;
mod vr;

use crate::i18n::tr;
use crate::state::{AppState, Client};
use crate::utils::unfocus_on_outside_click;
use advanced::advanced_section;
use adw::prelude::*;
use general::general_section;
use integration::integration_section;
use mods::mods_section;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use vr::vr_section;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SettingsCategory {
    General,
    Mods,
    Vr,
    Integration,
    Advanced,
}

impl SettingsCategory {
    pub const ALL: [Self; 5] = [
        Self::General,
        Self::Mods,
        Self::Vr,
        Self::Integration,
        Self::Advanced,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Mods => "Mods",
            Self::Vr => "VR",
            Self::Integration => "Integration",
            Self::Advanced => "Advanced",
        }
    }

    pub fn is_available(self, state: &AppState) -> bool {
        match self {
            Self::General | Self::Integration => true,
            Self::Mods | Self::Vr => state.selected_client() == Client::Player,
            Self::Advanced => state.advanced_settings_visible(),
        }
    }
}

#[derive(Clone)]
pub struct SettingsPage {
    pub widget: gtk::Widget,
    state: AppState,
    sections: Rc<Vec<(SettingsCategory, gtk::Box)>>,
    active: Rc<Cell<SettingsCategory>>,
    hidden_by_search: Rc<RefCell<Vec<gtk::Widget>>>,
    empty_state: gtk::Box,
    scrolled: gtk::ScrolledWindow,
    query: Rc<RefCell<String>>,
}

impl SettingsPage {
    pub fn set_category(&self, category: SettingsCategory) {
        self.active.set(category);
        self.query.borrow_mut().clear();
        self.refresh();
        self.scrolled.vadjustment().set_value(0.0);
    }

    pub fn set_filter(&self, query: &str) {
        *self.query.borrow_mut() = query.to_string();
        self.refresh();
    }

    pub fn active_category(&self) -> SettingsCategory {
        self.active.get()
    }

    pub fn refresh(&self) {
        if !self.active.get().is_available(&self.state) {
            self.active.set(SettingsCategory::General);
        }
        apply_view(
            &self.state,
            &self.sections,
            &self.active,
            &self.hidden_by_search,
            &self.query.borrow(),
            &self.empty_state,
        );
    }
}

pub fn settings_page(state: &AppState) -> SettingsPage {
    let sections = Rc::new(vec![
        (SettingsCategory::General, general_section(state)),
        (SettingsCategory::Mods, mods_section(state)),
        (SettingsCategory::Vr, vr_section(state)),
        (SettingsCategory::Integration, integration_section(state)),
        (SettingsCategory::Advanced, advanced_section(state)),
    ]);
    let active = Rc::new(Cell::new(SettingsCategory::General));
    let hidden_by_search: Rc<RefCell<Vec<gtk::Widget>>> = Rc::default();
    let empty_state = empty_search_box();

    let content_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(18)
        .margin_top(16)
        .margin_bottom(24)
        .build();
    for (_, section) in sections.iter() {
        content_box.append(section);
    }
    content_box.append(&empty_state);

    let scrolled = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::External)
        .vexpand(true)
        .hexpand(true)
        .child(&content_box)
        .build();
    unfocus_on_outside_click(&scrolled);

    let page = SettingsPage {
        widget: scrolled.clone().upcast(),
        state: state.clone(),
        sections,
        active,
        hidden_by_search,
        empty_state,
        scrolled,
        query: Rc::default(),
    };
    page.refresh();

    page
}

fn empty_search_box() -> gtk::Box {
    let box_ = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(4)
        .halign(gtk::Align::Fill)
        .css_classes(["card", "settings-empty"])
        .visible(false)
        .build();
    box_.append(
        &gtk::Label::builder()
            .label(tr("No matching settings found"))
            .css_classes(["heading"])
            .build(),
    );
    box_.append(
        &gtk::Label::builder()
            .label(tr(
                "Try searching for Runtime, Vulkan, FPS, MangoHud, or Discord",
            ))
            .css_classes(["dimmed", "caption"])
            .build(),
    );
    box_
}

fn apply_view(
    state: &AppState,
    sections: &[(SettingsCategory, gtk::Box)],
    active: &Cell<SettingsCategory>,
    hidden_by_search: &RefCell<Vec<gtk::Widget>>,
    raw_query: &str,
    empty_state: &gtk::Box,
) {
    for widget in hidden_by_search.borrow_mut().drain(..) {
        widget.set_visible(true);
    }

    let query = raw_query.trim().to_lowercase();
    if query.is_empty() {
        empty_state.set_visible(false);
        let current = active.get();
        for (category, section) in sections {
            section.set_visible(*category == current && category.is_available(state));
        }
        return;
    }

    let mut total_matches = 0;
    let mut hidden = hidden_by_search.borrow_mut();

    for (category, section) in sections {
        if !category.is_available(state) {
            section.set_visible(false);
            continue;
        }

        let mut groups = Vec::new();
        collect_groups(section.upcast_ref(), &mut groups);

        let mut cat_matches = 0;
        for group in groups {
            let group_title = group.title().to_string();
            let mut rows = Vec::new();
            collect_rows(group.upcast_ref(), &mut rows);

            let mut group_matches = 0;
            for row in rows {
                if row_matches(&row, &group_title, &query) {
                    if row.is_visible() {
                        group_matches += 1;
                    }
                } else if row.is_visible() {
                    row.set_visible(false);
                    hidden.push(row.upcast());
                }
            }

            if group_matches == 0 {
                if group.is_visible() {
                    group.set_visible(false);
                    hidden.push(group.upcast());
                }
            } else {
                cat_matches += group_matches;
            }
        }

        section.set_visible(cat_matches > 0);
        total_matches += cat_matches;
    }

    empty_state.set_visible(total_matches == 0);
}

fn collect_groups(widget: &gtk::Widget, out: &mut Vec<adw::PreferencesGroup>) {
    if let Ok(group) = widget.clone().downcast::<adw::PreferencesGroup>() {
        out.push(group);
        return;
    }
    let mut child = widget.first_child();
    while let Some(current) = child {
        collect_groups(&current, out);
        child = current.next_sibling();
    }
}

fn collect_rows(widget: &gtk::Widget, out: &mut Vec<adw::PreferencesRow>) {
    if let Ok(row) = widget.clone().downcast::<adw::PreferencesRow>() {
        out.push(row);
        return;
    }
    let mut child = widget.first_child();
    while let Some(current) = child {
        collect_rows(&current, out);
        child = current.next_sibling();
    }
}

fn row_matches(row: &adw::PreferencesRow, group_title: &str, query: &str) -> bool {
    if group_title.to_lowercase().contains(query) {
        return true;
    }
    if row.title().to_lowercase().contains(query) {
        return true;
    }
    if let Some(action) = row.downcast_ref::<adw::ActionRow>()
        && let Some(subtitle) = action.subtitle()
        && subtitle.to_lowercase().contains(query)
    {
        return true;
    }
    false
}
