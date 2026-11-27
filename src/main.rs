mod app;
mod backend;
mod command_line;
mod components;
mod i18n;
mod pages;
mod state;
mod utils;

use adw::{Application, prelude::*};
use app::Rovarch;
use gtk::{gio, glib};
use state::AppState;
use std::cell::OnceCell;
use std::rc::Rc;

const APP_ID: &str = "io.rovarch.app";

fn main() -> glib::ExitCode {
    if let Some(exit_code) = command_line::run_report_if_requested() {
        return exit_code;
    }

    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();
    command_line::add_options(&app);
    let launcher: Rc<OnceCell<Rc<Rovarch>>> = Rc::default();

    app.connect_startup(glib::clone!(
        #[strong]
        launcher,
        move |app| {
            let state = AppState::load();
            i18n::initialize(state.app_language());
            if i18n::is_rtl() {
                gtk::Widget::set_default_direction(gtk::TextDirection::Rtl);
            }
            state.connect_notify_local(Some("app-language"), |state, _| {
                if state.busy() || state.running() {
                    return;
                }
                state.save_pending_changes();
                backend::restart_to_apply_settings();
            });
            backend::initialize(&state);
            apply_rovarch_theme();
            app.connect_shutdown(glib::clone!(
                #[strong]
                state,
                move |_| {
                    state.save_pending_changes();
                    backend::finish_pending_update();
                }
            ));
            launcher.get_or_init(|| Rovarch::start(app, &state));
        }
    ));

    app.connect_command_line(glib::clone!(
        #[strong]
        launcher,
        move |_, command_line| {
            if let Some(launcher) = launcher.get() {
                match command_line::launch_requested_by(command_line) {
                    Some(request) => launcher.launch_directly(request),
                    None => launcher.activate(),
                }
            }
            glib::ExitCode::SUCCESS
        }
    ));

    app.connect_activate(move |_| {
        if let Some(launcher) = launcher.get() {
            launcher.activate();
        }
    });

    app.run()
}

fn apply_rovarch_theme() {
    utils::register_bundled_fonts();

    let display = gtk::gdk::Display::default().unwrap();

    let rovarch = gtk::CssProvider::new();
    rovarch.load_from_string(include_str!("styles/rovarch.css"));
    gtk::style_context_add_provider_for_display(
        &display,
        &rovarch,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    adw::StyleManager::default().set_color_scheme(adw::ColorScheme::ForceDark);
}
