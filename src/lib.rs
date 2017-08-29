/*!
A small Rust library to scrape useful data from [over.gg](https://www.over.gg/).

[over.gg](https://www.over.gg/) is a site that hosts articles, discussion,
team information, match results, and upcoming / live match information for the
competitive Overwatch scene. It unfortunately does not provide any kind of API
to access its extremely useful data, which is why this scraper has to exist.

## Development Status

This library currently only supports scraping a small amount of data from the
site. I may or may not add support for collecting more data as time and interest
allow. Contributions are certainly welcome!

I do not currently intend to publish this on crates.io as it is a very niche crate.

## Testing

There are currently two kinds of tests that this library employs: one that
grabs HTML from the live site and runs some assertions on the output, and
another that uses locally provided HTML files to ensure that extraction
of data from the HTML does not change. The local dataset is not staged in
the repo in an attempt to remain legal :), so you will have to provide your
own dataset in order to run the local test variety. This is currently as simple
as saving the [over.gg](https://www.over.gg/) page HTML to
`test_data/www.over.gg.html` via your web browser. If more data extraction gets
added in the future you will likely have to provide more pages.

## Rust Version Support

I only support the latest stable version of Rust; the library may or may not
compile on an older version.

## Cargo Features

* `http_client` toggles compilation of the `http_client` module.
* `derive-serde` toggles derivation of `serde`'s `Serialize` and `Deserialize`
traits for appropriate data structures, both in this crate and its dependencies.
* `test-local-data` is for the development of the library, enabling tests that
utilize local HTML files.

The `http_client` feature is enabled by default.

## Disclaimer

**Please use this library responsibly**. It is intended for occasional
programmatic retrieval of data from the [over.gg](https://www.over.gg/) site,
nothing more, and I do not endorse excessive scraping.
*/

#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate if_chain;
#[cfg(feature = "http-client")]
extern crate reqwest;
extern crate scraper;
extern crate chrono;
extern crate toml;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[cfg(all(feature = "test-local-data", test))]
extern crate serde_json;

pub mod error;
pub mod data_structs;
pub mod scrapers;
pub mod http_client;
#[cfg(test)]
#[cfg(feature = "test-local-data")]
mod test_utils;
