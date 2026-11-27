use crate::state::AppState;
use gtk::glib;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

const PULSE_EVERY: Duration = Duration::from_millis(100);

pub fn task_progress_bar(state: &AppState) -> adw::Bin {
    let bar = gtk::ProgressBar::builder()
        .css_classes(["task-bar"])
        .visible(false)
        .build();
    let pulsing: Rc<RefCell<Option<glib::SourceId>>> = Rc::default();

    state.follow_properties(
        &["busy", "task-progress"],
        glib::clone!(
            #[weak]
            bar,
            #[strong]
            pulsing,
            move |state| show_progress(&bar, &pulsing, state)
        ),
    );

    adw::Bin::builder().child(&bar).build()
}

fn show_progress(
    bar: &gtk::ProgressBar,
    pulsing: &Rc<RefCell<Option<glib::SourceId>>>,
    state: &AppState,
) {
    if !state.busy() {
        stop_pulsing(pulsing);
        bar.set_visible(false);
        bar.set_fraction(0.0);
        return;
    }

    bar.set_visible(true);
    let progress = state.task_progress();
    if progress > 0.0 {
        stop_pulsing(pulsing);
        bar.set_fraction(progress);
    } else {
        start_pulsing(bar, pulsing);
    }
}

fn start_pulsing(bar: &gtk::ProgressBar, pulsing: &Rc<RefCell<Option<glib::SourceId>>>) {
    if pulsing.borrow().is_some() {
        return;
    }
    let bar = bar.downgrade();
    let source = glib::timeout_add_local(PULSE_EVERY, move || {
        let Some(bar) = bar.upgrade() else {
            return glib::ControlFlow::Break;
        };
        bar.pulse();
        glib::ControlFlow::Continue
    });
    *pulsing.borrow_mut() = Some(source);
}

fn stop_pulsing(pulsing: &Rc<RefCell<Option<glib::SourceId>>>) {
    if let Some(source) = pulsing.borrow_mut().take() {
        source.remove();
    }
}
