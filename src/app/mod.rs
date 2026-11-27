use crate::backend::{self, LaunchRequest};
use crate::components::close_roblox_dialog::confirm_closing_roblox;
use crate::components::launch_window::launch_window;
use crate::components::play_action::PlayAction;
use crate::components::tray::{TrayIcon, TrayRequest};
use crate::state::{AppState, LaunchMode};
use adw::{Application, ApplicationWindow, ViewStack, prelude::*};
use gtk::glib;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

mod window;

use window::{pages, window};

const PROPERTIES_THAT_DECIDE_STAYING_OPEN: [&str; 5] = [
    "busy",
    "running",
    "run-in-background",
    "launch-mode",
    "hide-launcher-while-playing",
];
const PROPERTIES_THAT_DECIDE_TRAY_ICON: [&str; 2] = ["run-in-background", "launch-mode"];

#[derive(Clone, Copy)]
enum Page {
    Home,
    Servers,
    Settings,
}

impl Page {
    fn name(self) -> &'static str {
        match self {
            Self::Home => "home",
            Self::Servers => "servers",
            Self::Settings => "settings",
        }
    }
}

pub struct Rovarch {
    app: Application,
    state: AppState,
    window: ApplicationWindow,
    launch_window: ApplicationWindow,
    pages: ViewStack,
    tray_icon: RefCell<Option<TrayIcon>>,
    launched_directly: Cell<bool>,
    quits_once_roblox_exits: Cell<bool>,
}

impl Rovarch {
    pub fn start(app: &Application, state: &AppState) -> Rc<Self> {
        let (pages, settings) = pages(state);
        let window = window(app, state, &pages, &settings);
        let launcher = Rc::new(Self {
            app: app.clone(),
            state: state.clone(),
            window,
            launch_window: launch_window(app, state),
            pages,
            tray_icon: RefCell::new(None),
            launched_directly: Cell::new(false),
            quits_once_roblox_exits: Cell::new(false),
        });

        launcher.window.connect_visible_notify(glib::clone!(
            #[weak]
            launcher,
            move |_| launcher.quit_if_nothing_keeps_it_open()
        ));
        launcher.window.connect_close_request(glib::clone!(
            #[weak]
            launcher,
            #[upgrade_or]
            glib::Propagation::Proceed,
            move |_| launcher.close_window()
        ));
        state.connect_running_notify(glib::clone!(
            #[weak]
            launcher,
            move |_| launcher.adjust_window_to_running_roblox()
        ));
        state.connect_properties_notify(
            &PROPERTIES_THAT_DECIDE_STAYING_OPEN,
            glib::clone!(
                #[weak]
                launcher,
                move |_| launcher.quit_if_nothing_keeps_it_open()
            ),
        );
        state.follow_properties(
            &PROPERTIES_THAT_DECIDE_TRAY_ICON,
            glib::clone!(
                #[weak]
                launcher,
                move |_| launcher.show_tray_icon_when_wanted()
            ),
        );

        launcher
    }

    pub fn activate(&self) {
        if self.state.launch_mode() == LaunchMode::LaunchRoblox {
            self.launch_directly(LaunchRequest::Roblox { link: None });
        } else {
            self.show(Page::Home);
        }
    }

    pub fn launch_directly(&self, request: LaunchRequest) {
        if self.state.running() || self.state.busy() {
            self.show(Page::Home);
            return;
        }

        self.launched_directly.set(true);
        self.state.set_selected_client(request.client());
        backend::launch(&self.state, request);
        if self.state.busy() {
            self.launch_window.present();
        }
    }

    fn show(&self, page: Page) {
        self.launched_directly.set(false);
        self.pages.set_visible_child_name(page.name());
        self.window.present();
    }

    fn handle(self: &Rc<Self>, request: TrayRequest) {
        match request {
            TrayRequest::OpenHome => self.show(Page::Home),
            TrayRequest::OpenSettings => self.show(Page::Settings),
            TrayRequest::PerformPlayAction => {
                let action = PlayAction::for_client(&self.state, self.state.current_client());
                if action.is_available(&self.state) {
                    action.perform(&self.state);
                }
            }
            TrayRequest::Quit => self.confirm_quit(),
        }
    }

    fn close_window(self: &Rc<Self>) -> glib::Propagation {
        if self.state.running() && !self.should_keep_running_without_window() {
            self.confirm_quit();
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    }

    fn confirm_quit(self: &Rc<Self>) {
        if !self.state.running() {
            self.app.quit();
            return;
        }
        self.window.present();
        let launcher = Rc::downgrade(self);
        confirm_closing_roblox(&self.window, move || {
            if let Some(launcher) = launcher.upgrade() {
                launcher.quit();
            }
        });
    }

    fn quit(&self) {
        if !self.state.running() {
            self.app.quit();
            return;
        }
        self.quits_once_roblox_exits.set(true);
        self.window.set_visible(false);
        backend::stop(&self.state);
    }

    fn adjust_window_to_running_roblox(&self) {
        if self.quits_once_roblox_exits.get() && !self.state.running() {
            self.app.quit();
            return;
        }
        if self.state.running() && self.state.hide_launcher_while_playing() {
            self.window.set_visible(false);
        } else if !self.state.running() && self.state.open_after_exit() {
            self.show(Page::Home);
        }
    }

    fn wants_tray_icon(&self) -> bool {
        self.state.run_in_background() || self.state.launch_mode() == LaunchMode::LaunchRoblox
    }

    fn should_keep_running_without_window(&self) -> bool {
        let can_be_reopened_from_tray = self.tray_icon.borrow().is_some();
        let launches_without_window =
            self.state.launch_mode() == LaunchMode::LaunchRoblox || self.launched_directly.get();
        let is_launching_right_away =
            launches_without_window && (self.state.busy() || self.state.running());
        let is_hidden_while_playing =
            self.state.hide_launcher_while_playing() && self.state.running();
        let is_closing_roblox_first = self.quits_once_roblox_exits.get();

        (self.state.run_in_background() && can_be_reopened_from_tray)
            || is_launching_right_away
            || is_hidden_while_playing
            || is_closing_roblox_first
    }

    fn quit_if_nothing_keeps_it_open(&self) {
        if !self.window.is_visible() && !self.should_keep_running_without_window() {
            self.app.quit();
        }
    }

    fn show_tray_icon_when_wanted(self: &Rc<Self>) {
        let wants_tray_icon = self.wants_tray_icon();
        let has_tray_icon = self.tray_icon.borrow().is_some();

        if wants_tray_icon && !has_tray_icon {
            let launcher = Rc::downgrade(self);
            let tray_icon = TrayIcon::show(&self.state, move |request| {
                if let Some(launcher) = launcher.upgrade() {
                    launcher.handle(request);
                }
            });
            match tray_icon {
                Ok(tray_icon) => *self.tray_icon.borrow_mut() = Some(tray_icon),
                Err(error) => eprintln!("Failed to show the tray icon: {error}"),
            }
        } else if !wants_tray_icon && has_tray_icon {
            self.tray_icon.borrow_mut().take();
        }
    }
}
