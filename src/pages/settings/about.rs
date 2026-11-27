use crate::components::icon::{app_icon, dratiux_logo};
use crate::i18n::tr;
use crate::utils::floating_window;
use adw::prelude::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const WEBSITE: &str = "https://github.com/RovarchHQ/Rovarch";
const ROVARCH_CORE: &str = "https://github.com/RovarchHQ/rovarch-core";
const DRATIUX_WEBSITE: &str = "https://dratiux.app";

pub fn show_about(parent: &impl IsA<gtk::Widget>) {
    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .spacing(8)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(32)
        .margin_end(32)
        .build();

    let icon = app_icon(76);
    icon.set_margin_bottom(6);
    content.append(&icon);

    let name = gtk::Label::builder()
        .label("Rovarch")
        .css_classes(["title-1"])
        .build();
    content.append(&name);

    let version = gtk::Label::builder()
        .label(format!("v{VERSION}"))
        .css_classes(["version-badge"])
        .halign(gtk::Align::Center)
        .build();
    content.append(&version);

    let comments = gtk::Label::builder()
        .label(tr(
            "A Linux launcher for Rovarch Play and Rovarch Studio, powered by Rovarch CORE.",
        ))
        .wrap(true)
        .justify(gtk::Justification::Center)
        .max_width_chars(44)
        .use_markup(false)
        .css_classes(["dimmed"])
        .margin_top(8)
        .build();
    content.append(&comments);

    let links = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .halign(gtk::Align::Center)
        .spacing(8)
        .margin_top(8)
        .build();
    links.append(&link_button("Rovarch", WEBSITE));
    links.append(&link_button("Rovarch CORE", ROVARCH_CORE));
    content.append(&links);

    content.append(&rule());

    let signature_content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .halign(gtk::Align::Center)
        .spacing(6)
        .build();
    signature_content.append(&dratiux_logo(104, 16));
    signature_content.append(
        &gtk::Label::builder()
            .label(tr("Created with care by dratiux."))
            .css_classes(["about-credit"])
            .justify(gtk::Justification::Center)
            .build(),
    );

    let signature_button = gtk::Button::builder()
        .css_classes(["flat", "about-signature-btn"])
        .halign(gtk::Align::Center)
        .child(&signature_content)
        .build();
    signature_button.set_cursor_from_name(Some("pointer"));
    signature_button.connect_clicked(|button| {
        let window = button.root().and_downcast::<gtk::Window>();
        gtk::UriLauncher::new(DRATIUX_WEBSITE).launch(
            window.as_ref(),
            gtk::gio::Cancellable::NONE,
            |_| {},
        );
    });

    content.append(&signature_button);
    content.append(
        &gtk::Label::builder()
            .label(tr("Released under GPL-3.0."))
            .css_classes(["dimmed", "caption"])
            .justify(gtk::Justification::Center)
            .margin_top(2)
            .build(),
    );

    let scrolled = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::External)
        .propagate_natural_height(true)
        .child(&content)
        .build();

    let window = floating_window(parent, &tr("About Rovarch"), 460, 480, false, &scrolled);
    window.add_css_class("about");
    window.present();
    gtk::prelude::GtkWindowExt::set_focus(&window, None::<&gtk::Widget>);
}

fn rule() -> gtk::Separator {
    gtk::Separator::builder()
        .orientation(gtk::Orientation::Horizontal)
        .margin_top(10)
        .margin_bottom(10)
        .build()
}

fn link_button(label: &str, uri: &str) -> gtk::LinkButton {
    let button = gtk::LinkButton::builder()
        .label(label)
        .uri(uri)
        .halign(gtk::Align::Center)
        .build();
    button.add_css_class("flat");
    button
}
