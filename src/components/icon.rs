use crate::state::Client;
use crate::utils::{
    APP_ICON_PNG, DRATIUX_LOGO_PNG, PLAY_LOGO_PNG, PLAY_LOGO_SVG, STUDIO_LOGO_PNG, STUDIO_LOGO_SVG,
};
use gtk::gdk;
use gtk::gdk_pixbuf;
use gtk::glib;
use gtk::prelude::*;
use std::path::PathBuf;

const BRAND_CACHE_DIRECTORY: &str = "branding";

pub fn app_icon_texture() -> Result<gdk::Texture, glib::Error> {
    gdk::Texture::from_bytes(&glib::Bytes::from_static(APP_ICON_PNG))
}

pub fn app_icon(size: i32) -> gtk::Image {
    let icon = gtk::Image::new();
    match app_icon_texture() {
        Ok(texture) => icon.set_paintable(Some(&texture)),
        Err(error) => eprintln!("Failed to load the app icon: {error}"),
    }
    icon.set_pixel_size(size);
    icon.add_css_class("app-icon");

    icon
}

pub fn header_emblem(size: i32) -> gtk::Image {
    client_logo(Client::Player, size)
}

pub fn dratiux_logo(width: i32, height: i32) -> gtk::Picture {
    let picture = gtk::Picture::builder()
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .can_shrink(false)
        .build();
    let stream =
        gtk::gio::MemoryInputStream::from_bytes(&glib::Bytes::from_static(DRATIUX_LOGO_PNG));
    match gdk_pixbuf::Pixbuf::from_stream_at_scale(
        &stream,
        width,
        height,
        true,
        gtk::gio::Cancellable::NONE,
    ) {
        Ok(pixbuf) => picture.set_paintable(Some(&gdk::Texture::for_pixbuf(&pixbuf))),
        Err(error) => eprintln!("Failed to load the dratiux logo: {error}"),
    }
    picture
}

pub fn client_logo(client: Client, size: i32) -> gtk::Image {
    let image = gtk::Image::new();
    match client_logo_pixbuf(client, size) {
        Ok(pixbuf) => image.set_paintable(Some(&gdk::Texture::for_pixbuf(&pixbuf))),
        Err(error) => {
            eprintln!("Couldn't load the SVG {client:?} logo, falling back to PNG: {error}");
            match client_logo_texture(client) {
                Ok(texture) => image.set_paintable(Some(&texture)),
                Err(error) => eprintln!("Failed to load the {client:?} logo: {error}"),
            }
        }
    }
    image.set_pixel_size(size);
    image.add_css_class("app-icon");

    image
}

fn logo_source(client: Client) -> (&'static str, &'static [u8], &'static [u8]) {
    match client {
        Client::Player => ("rovarch-play-emblem.svg", PLAY_LOGO_SVG, PLAY_LOGO_PNG),
        Client::Studio => (
            "rovarch-studio-emblem.svg",
            STUDIO_LOGO_SVG,
            STUDIO_LOGO_PNG,
        ),
    }
}

fn client_logo_pixbuf(client: Client, size: i32) -> Result<gdk_pixbuf::Pixbuf, glib::Error> {
    let (file_name, logo_svg, _) = logo_source(client);
    let file = brand_file(file_name);
    if let Some(directory) = file.parent() {
        std::fs::create_dir_all(directory).ok();
    }
    if std::fs::read(&file).ok().as_deref() != Some(logo_svg) {
        std::fs::write(&file, logo_svg).ok();
    }
    gdk_pixbuf::Pixbuf::from_file_at_size(&file, size, size)
}

fn client_logo_texture(client: Client) -> Result<gdk::Texture, glib::Error> {
    let (_, _, logo_png) = logo_source(client);
    gdk::Texture::from_bytes(&glib::Bytes::from_static(logo_png))
}

fn brand_directory() -> PathBuf {
    glib::user_cache_dir()
        .join("rovarch")
        .join(BRAND_CACHE_DIRECTORY)
}

fn brand_file(file_name: &str) -> PathBuf {
    brand_directory().join(file_name)
}
