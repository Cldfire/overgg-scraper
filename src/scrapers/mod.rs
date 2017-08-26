//! Various modules for getting data from the site's HTML.

pub mod main_page;

use scraper::Selector;

fn sel<S: AsRef<str>>(sel: S) -> Selector {
    Selector::parse(sel.as_ref()).unwrap()
}
