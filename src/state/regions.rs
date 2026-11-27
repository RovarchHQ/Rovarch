use gtk::glib;
use serde::{Deserialize, Serialize};

const COUNTRY_AND_CITY_SEPARATOR: char = ':';

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Region {
    pub country_name: String,
    pub country_code: String,
    pub city: Option<String>,
}

impl Region {
    pub fn key(&self) -> String {
        let code = self.country_code.trim();
        match &self.city {
            Some(city) => format!("{code}{COUNTRY_AND_CITY_SEPARATOR}{}", city.trim()),
            None => code.to_owned(),
        }
    }

    pub fn label(&self) -> String {
        let name = self.country_name.trim();
        match &self.city {
            Some(city) => format!("{name} · {}", city.trim()),
            None => name.to_owned(),
        }
    }
}

pub fn country_and_city_of(region_key: &str) -> (&str, Option<&str>) {
    match region_key.split_once(COUNTRY_AND_CITY_SEPARATOR) {
        Some((country, city)) => (country, Some(city)),
        None => (region_key, None),
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, glib::Boxed)]
#[boxed_type(name = "RovarchRegions")]
pub struct Regions(pub Vec<Region>);

impl Regions {
    pub fn label_of(&self, region_key: &str) -> String {
        self.0
            .iter()
            .find(|region| region.key() == region_key)
            .map(Region::label)
            .unwrap_or_else(|| region_key.to_owned())
    }
}
