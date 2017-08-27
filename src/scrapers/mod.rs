//! Various modules for getting data from the site's HTML.

pub mod main_page;

use scraper::Selector;
use std::collections::HashMap;

fn sel<S: AsRef<str>>(sel: S) -> Selector {
    Selector::parse(sel.as_ref()).unwrap()
}

fn load_sels<S: AsRef<str>>(from: S) -> HashMap<String, Selector> {
    let strings: HashMap<String, String> =
        ::toml::from_str(from.as_ref()).unwrap();

    strings.into_iter().map(|(key, val)| (key, sel(val))).collect()
}
