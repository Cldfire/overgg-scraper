//! A quick, built-in way to grab HTML from the live site.
//!
//! This module can be disabled by building this crate without default features.

use crate::error::*;
use reqwest::{Client, IntoUrl};
use crate::scrapers::main_page::MainPageScraper;
use std::io::Read;

/// A quick, built-in way to grab HTML from the live site.
pub struct Downloader {
    client: Client
}

impl Downloader {
    /// Create a HTTP client with reqwest's default `Client` config.
    #[inline]
    pub fn new() -> Self {
        Self {
            client: Client::new()
        }
    }

    /// Provide your own client for use by this struct.
    ///
    /// This means you can configure the client as you wish while still doing
    /// less work than re-implementing the functionality provided by this struct.
    #[inline]
    pub fn with_client(client: Client) -> Self {
        Self {
            client
        }
    }

    /// Obtain a scraper for the main page (https://www.over.gg/).
    #[inline]
    pub fn main_page(&self) -> Result<MainPageScraper> {
        Ok(self.get_string("https://www.over.gg/")?.into())
    }

    /// Helper to get the HTML of the given URL.
    ///
    /// Panics if the given `url` cannot be parsed.
    #[inline]
    fn get_string<U: IntoUrl>(&self, url: U) -> Result<String> {
        let mut resp = self.client.get(url).send()?;
        let status = resp.status();

        if !status.is_success() {
            bail!(ErrorKind::NonSuccessStatus(status));
        }

        let mut content = String::new();
        resp.read_to_string(&mut content)?;

        Ok(content)
    }
}
