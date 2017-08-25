# overgg-scraper

A small Rust library to scrape useful data from [over.gg](https://www.over.gg/), a site that hosts articles, discussion, team information, match results, and upcoming / live match information for the competitive Overwatch scene.

```rust
let dlr = Downloader::new()?;
let scraper = dlr.main_page()?;

let completed_matches = scraper.matches_brief(Completed)?;
let upcoming_matches = scraper.matches_brief(InFuture)?;
let live_matches = scraper.matches_brief(Live)?;
```

The site unfortunately does not provide any kind of API to access its extremely useful data, which is why this scraper has to exist.

## Development Status

This library currently only supports scraping a small amount of data from the site. I may or may not add support for collecting more data as time and interest allow. Contributions are certainly welcome!

I do not currently intend to publish this on crates.io as it is a very niche crate.

## Rust Version Support

I only support the latest stable version of Rust; the library may or may not compile on an older version.

## Cargo Features

* `http_client` toggles compilation of the `http_client` module.
* `derive-serde` toggles derivation of `serde`'s `Serialize` and `Deserialize` traits for appropriate data structures, both in this crate and its dependencies.
* `test-local-data` is for the development of the library, enabling tests that utilize local HTML files.

The `http_client` feature is enabled by default.

## Disclaimer

**Please use this library responsibly**. It is intended for occasional programmatic retrieval of data from the [over.gg](https://www.over.gg/) site, nothing more, and I do not endorse excessive scraping.

## License

Licensed under either of

* Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
