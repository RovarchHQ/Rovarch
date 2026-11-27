use super::environment_variables::environment_variables_group;
use super::fast_flags::fast_flags_section;
use super::launch_wrapper::launch_wrapper_group;
use crate::i18n::tr;
use crate::state::AppState;
use adw::prelude::*;

pub fn advanced_section(state: &AppState) -> gtk::Box {
    let section = gtk::Box::new(gtk::Orientation::Vertical, 18);

    let warning = gtk::Label::builder()
        .label(tr("⚠️ Advanced settings\n\
             These options can break Rovarch or Roblox if misconfigured. \
             Only change them if you know what you're doing."))
        .wrap(true)
        .justify(gtk::Justification::Center)
        .css_classes(["dim-label", "caption"])
        .build();
    section.append(&warning);
    section.append(&launch_wrapper_group(state));
    section.append(&environment_variables_group(state));
    section.append(&fast_flags_section(state));

    section
}
