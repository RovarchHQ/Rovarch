use super::Page;
use crate::components::error_dialog::show_errors_from;
use crate::components::grid_backdrop::grid_backdrop;
use crate::components::icon::header_emblem;
use crate::components::task_progress::task_progress_bar;
use crate::components::update_banner::update_banner;
use crate::i18n::tr;
use crate::pages::home::home_page;
use crate::pages::servers::servers_page;
use crate::pages::settings::{SettingsCategory, SettingsPage, settings_page, show_about};
use crate::state::{AppState, Client};
use crate::utils::show_only_while_roblox_is_selected;
use adw::{Application, ApplicationWindow, ToolbarView, ViewStack, prelude::*};
use gtk::glib;
use std::rc::Rc;

pub(super) fn pages(state: &AppState) -> (ViewStack, SettingsPage) {
    let pages = ViewStack::new();
    pages.set_hexpand(true);
    pages.set_vexpand(true);

    pages.add_titled(
        &home_page(state),
        Some(Page::Home.name()),
        tr("Home").as_str(),
    );
    let servers = pages.add_titled(
        &servers_page(state),
        Some(Page::Servers.name()),
        tr("Servers").as_str(),
    );
    show_only_while_roblox_is_selected(&servers, state);

    let settings = settings_page(state);
    pages.add_titled(
        &settings.widget,
        Some(Page::Settings.name()),
        tr("Settings").as_str(),
    );

    (pages, settings)
}

fn header_bar() -> adw::HeaderBar {
    let brand = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(10)
        .valign(gtk::Align::Center)
        .build();

    let title = gtk::Label::builder()
        .label("Rovarch")
        .css_classes(["header-brand-title"])
        .build();

    let version = gtk::Label::builder()
        .label(format!("v{}", env!("CARGO_PKG_VERSION")))
        .css_classes(["version-badge"])
        .valign(gtk::Align::Center)
        .build();

    brand.append(&header_emblem(22));
    brand.append(&title);
    brand.append(&version);

    let header = adw::HeaderBar::builder()
        .title_widget(&gtk::Box::new(gtk::Orientation::Horizontal, 0))
        .show_start_title_buttons(false)
        .show_end_title_buttons(false)
        .build();
    header.pack_start(&brand);
    header
}

fn nav_item(label_text: &str, subitem: bool) -> gtk::Button {
    let label = gtk::Label::builder()
        .label(label_text)
        .halign(gtk::Align::Start)
        .build();
    let classes = if subitem {
        ["settings-nav-item", "sidebar-subitem"]
    } else {
        ["settings-nav-item", "sidebar-item"]
    };
    gtk::Button::builder()
        .css_classes(classes)
        .child(&label)
        .build()
}

fn set_active(button: &gtk::Button, active: bool) {
    if active {
        button.add_css_class("active");
    } else {
        button.remove_css_class("active");
    }
}

fn unified_sidebar(state: &AppState, pages: &ViewStack, settings: &SettingsPage) -> gtk::Box {
    let search = gtk::Entry::builder()
        .placeholder_text(tr("Filter settings..."))
        .css_classes(["settings-search"])
        .visible(false)
        .build();

    let home_btn = nav_item(&tr("Home"), false);
    let play_btn = nav_item(&tr("Rovarch Play"), true);
    let studio_btn = nav_item(&tr("Rovarch Studio"), true);

    let home_sub = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(2)
        .css_classes(["sidebar-sublist"])
        .build();
    home_sub.append(&play_btn);
    home_sub.append(&studio_btn);

    let servers_btn = nav_item(&tr("Servers"), false);
    show_only_while_roblox_is_selected(&servers_btn, state);

    let settings_btn = nav_item(&tr("Settings"), false);
    let settings_sub = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(2)
        .css_classes(["sidebar-sublist"])
        .visible(false)
        .build();

    let category_buttons: Rc<Vec<(SettingsCategory, gtk::Button)>> = Rc::new(
        SettingsCategory::ALL
            .iter()
            .map(|&cat| {
                let btn = nav_item(&tr(cat.title()), true);
                settings_sub.append(&btn);
                (cat, btn)
            })
            .collect(),
    );

    let nav_card = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(2)
        .css_classes(["settings-nav-card"])
        .build();
    nav_card.append(&home_btn);
    nav_card.append(&home_sub);
    nav_card.append(&servers_btn);
    nav_card.append(&settings_btn);
    nav_card.append(&settings_sub);

    let about_button = gtk::Button::builder().label(tr("About Rovarch")).build();
    about_button.connect_clicked(show_about);

    let sidebar = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .width_request(210)
        .hexpand(false)
        .halign(gtk::Align::Start)
        .valign(gtk::Align::Start)
        .margin_top(16)
        .margin_bottom(24)
        .css_classes(["settings-sidebar"])
        .build();
    sidebar.append(&search);
    sidebar.append(&nav_card);
    sidebar.append(&about_button);

    let sync_sidebar = {
        let state = state.clone();
        let pages = pages.clone();
        let settings = settings.clone();
        let search = search.clone();
        let home_btn = home_btn.clone();
        let play_btn = play_btn.clone();
        let studio_btn = studio_btn.clone();
        let servers_btn = servers_btn.clone();
        let settings_btn = settings_btn.clone();
        let settings_sub = settings_sub.clone();
        let category_buttons = category_buttons.clone();

        Rc::new(move || {
            let page_name = pages.visible_child_name();
            let current = page_name.as_deref();
            let is_home = current == Some(Page::Home.name());
            let is_servers = current == Some(Page::Servers.name());
            let is_settings = current == Some(Page::Settings.name());
            let client = state.selected_client();

            if is_servers && client != Client::Player {
                pages.set_visible_child_name(Page::Home.name());
                return;
            }

            set_active(&home_btn, is_home);
            set_active(&play_btn, is_home && client == Client::Player);
            set_active(&studio_btn, is_home && client == Client::Studio);
            set_active(&servers_btn, is_servers);
            set_active(&settings_btn, is_settings);

            search.set_visible(is_settings);
            settings_sub.set_visible(is_settings);

            settings.refresh();
            let active_cat = settings.active_category();
            let searching = !search.text().trim().is_empty();
            for (cat, btn) in category_buttons.iter() {
                btn.set_visible(cat.is_available(&state));
                set_active(btn, is_settings && !searching && *cat == active_cat);
            }
        })
    };

    home_btn.connect_clicked(glib::clone!(
        #[weak]
        pages,
        move |_| pages.set_visible_child_name(Page::Home.name())
    ));
    play_btn.connect_clicked(glib::clone!(
        #[strong]
        state,
        #[weak]
        pages,
        move |_| {
            state.set_selected_client(Client::Player);
            pages.set_visible_child_name(Page::Home.name());
        }
    ));
    studio_btn.connect_clicked(glib::clone!(
        #[strong]
        state,
        #[weak]
        pages,
        move |_| {
            state.set_selected_client(Client::Studio);
            pages.set_visible_child_name(Page::Home.name());
        }
    ));
    servers_btn.connect_clicked(glib::clone!(
        #[weak]
        pages,
        move |_| pages.set_visible_child_name(Page::Servers.name())
    ));
    settings_btn.connect_clicked(glib::clone!(
        #[weak]
        pages,
        move |_| pages.set_visible_child_name(Page::Settings.name())
    ));

    for (cat, btn) in category_buttons.iter() {
        let cat = *cat;
        btn.connect_clicked(glib::clone!(
            #[strong]
            settings,
            #[strong]
            sync_sidebar,
            #[weak]
            pages,
            #[weak]
            search,
            move |_| {
                if !search.text().is_empty() {
                    search.set_text("");
                }
                settings.set_category(cat);
                pages.set_visible_child_name(Page::Settings.name());
                sync_sidebar();
            }
        ));
    }

    search.connect_changed(glib::clone!(
        #[strong]
        settings,
        #[strong]
        sync_sidebar,
        move |search| {
            settings.set_filter(&search.text());
            sync_sidebar();
        }
    ));

    pages.connect_visible_child_name_notify(glib::clone!(
        #[strong]
        sync_sidebar,
        move |_| sync_sidebar()
    ));
    state.follow_properties(
        &["selected-client", "advanced-settings-visible"],
        glib::clone!(
            #[strong]
            sync_sidebar,
            move |_| sync_sidebar()
        ),
    );

    sync_sidebar();
    sidebar
}

pub(super) fn window(
    app: &Application,
    state: &AppState,
    pages: &ViewStack,
    settings: &SettingsPage,
) -> ApplicationWindow {
    let split = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(24)
        .vexpand(true)
        .build();
    split.append(&unified_sidebar(state, pages, settings));
    split.append(pages);

    let clamp = adw::Clamp::builder()
        .maximum_size(1280)
        .tightening_threshold(1280)
        .margin_start(16)
        .margin_end(16)
        .child(&split)
        .build();

    let toolbar_view = ToolbarView::new();
    toolbar_view.add_top_bar(&header_bar());
    toolbar_view.add_top_bar(&task_progress_bar(state));
    toolbar_view.add_top_bar(&update_banner(state));
    toolbar_view.set_content(Some(&clamp));

    let backdrop = gtk::Overlay::builder().child(&grid_backdrop()).build();
    backdrop.add_overlay(&toolbar_view);
    backdrop.set_measure_overlay(&toolbar_view, true);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Rovarch")
        .default_width(1060)
        .default_height(740)
        .hide_on_close(true)
        .content(&backdrop)
        .build();

    state.connect_error_message_notify(glib::clone!(
        #[weak]
        window,
        move |_| window.present()
    ));
    show_errors_from(state, &window);

    window
}
