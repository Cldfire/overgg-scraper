/*!
A small Rust library to scrape useful data from [over.gg](https://www.over.gg/).

[over.gg](https://www.over.gg/) is a site that hosts articles, discussion,
team information, match results, and upcoming / live match information for the
competitive Overwatch scene. It unfortunately does not provide any kind of API
to access its extremely useful data, which is why this scraper has to exist.

This library currently only supports scraping a small amount of data from the
site. I may or may not add support for collecting more data as time and interest
allow. Contributions are certainly welcome!

There are currently two kinds of tests that this library employs: one that
grabs HTML from the live site and runs some assertions on the output, and
another that uses locally provided HTML files to ensure that extraction
of data from the HTML does not change. The local dataset is not staged in
the repo in an attempt to remain legal :), so you will have to provide your
own dataset in order to run the local test variety. This is currently as simple
as saving the [over.gg](https://www.over.gg/) page HTML to
`test_data/www.over.gg.html` via your web browser. If more data extraction gets
added in the future you will likely have to provide more pages.

Finally, please use this library responsibly. It is intended for occasional
programmatic retrieval of data from the [over.gg](https://www.over.gg/) site and
nothing more.
*/

#[macro_use]
extern crate error_chain;
extern crate reqwest;
extern crate scraper;
extern crate chrono;
#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;
#[cfg(feature = "serde")]
#[cfg(test)]
extern crate serde_json;

pub mod error;
pub mod data_structs;
#[cfg(test)]
#[cfg(feature = "test-local-data")]
mod test_utils;

use error::*;
use reqwest::{Client, IntoUrl};
use scraper::{Html, Selector};
use chrono::{Utc, TimeZone, LocalResult};
use data_structs::{MatchBrief, MatchBriefInfo, MatchBriefType};
use data_structs::MatchBriefType::*;
use std::io::Read;

#[cfg(feature = "test-local-data")]
const TEST_DATA_MAIN_PAGE: &'static str = include_str!("../test_data/www.over.gg.html");

pub struct Scraper {
    client: Client
}

impl Scraper {
    /// Get a scraper with reqwest's default `Client` config.
    #[inline]
    pub fn new() -> Result<Scraper> {
        Ok(Scraper {
            client: Client::new()?
        })
    }

    /// Get a scraper that will use the `Client` you provide.
    ///
    /// This means you can configure the client as you wish.
    #[inline]
    pub fn with_client(client: Client) -> Scraper {
        Scraper {
            client
        }
    }

    /// Gets the main page (https://www.over.gg/) and extracts match info for all
    /// of the matches of the the given `MatchBriefType`.
    #[inline]
    pub fn matches_brief(&self, _type: MatchBriefType) -> Result<Vec<MatchBrief>> {
        let html;

        #[cfg(feature = "test-local-data")] { html = TEST_DATA_MAIN_PAGE.to_string(); }
        #[cfg(not(feature = "test-local-data"))] {
            html = self.get_string("https://www.over.gg/")?;
        }

        let doc = Html::parse_document(&html);
        let mut matches_info = vec![];

        // TODO: Get this out of the code and into a config file
        let matches_sel = 
            Selector::parse("div.wf-module.wf-card.mod-home-matches").unwrap();
        let header_sel = Selector::parse("div.wf-module-header").unwrap();
        let match_sel = Selector::parse("a.wf-module-item.mod-match").unwrap();
        let live_sel = Selector::parse("div.h-match-eta.mod-live").unwrap();
        let event_name_sel = Selector::parse("div.h-match-preview-event").unwrap();
        let event_series_sel = Selector::parse("div.h-match-preview-series").unwrap();
        let teams_sel = Selector::parse("div.h-match-team").unwrap();
        let team_name_sel = Selector::parse("div.h-match-team-name").unwrap();
        let team_score_sel = Selector::parse("div.h-match-team-score.mod-count").unwrap();
        let match_scheduled_time_sel = Selector::parse("div.h-match-preview-time").unwrap();
        let timestamp_attr_name = "data-utc-ts";

        // First we get the lists of upcoming / completed matches
        for list in doc.select(&matches_sel) {
            let header_text = match list.select(&header_sel).next() {
                Some(elem) => elem.text().collect::<String>(),
                None => bail!(ErrorKind::ExtractionError(html))
            };
            
            // Now we narrow down to the list that contains the match type we want
            if header_text.trim() == String::from(_type) {
                // Now we get the match type we want
                let matches: Vec<scraper::ElementRef> = 
                    list.select(&match_sel).filter(|e| {
                        // If we want live matches, we need to do some filtering
                        if _type == Live {
                            match e.select(&live_sel).next() {
                                Some(_) => true,
                                None => false
                            }
                        // If we want future matches, we need to do filtering too
                        } else if _type == InFuture {
                            match e.select(&live_sel).next() {
                                Some(_) => false,
                                None => true
                            }
                        // No filtering is required for completed matches
                        } else {
                            true
                        }
                    }).collect();

                // Finally we get information for each match
                for _match in matches {
                    let mut match_info = MatchBriefInfo::default();

                    // Event name
                    if let Some(elem) = _match.select(&event_name_sel).next() {
                        match_info.event.name = elem.text().collect::<String>().trim().into();
                    }

                    // Event series
                    if let Some(elem) = _match.select(&event_series_sel).next() {
                        match_info.event.series = elem.text().collect::<String>().trim().into();
                    }

                    let mut teams = _match.select(&teams_sel);

                    for i in 0..2 {
                        if let Some(team) = teams.next() {
                            // Team name
                            if let Some(elem) = team.select(&team_name_sel).next() {
                                match_info.teams[i].name = elem.text().collect::<String>().trim().into();
                            }

                            // Team won maps count
                            if let Some(elem) = team.select(&team_score_sel).next() {
                                match_info.teams[i].maps_won = match elem.text()
                                                                         .collect::<String>()
                                                                         .trim()
                                                                         .parse() {
                                    Ok(val) => Some(val),
                                    Err(_) => None
                                }
                            }
                        }
                    }

                    // Scheduled match time
                    if let Some(elem) = _match.select(&match_scheduled_time_sel).next() {
                        if let Some(val) = elem.value().attr(timestamp_attr_name) {
                            if let Ok(timestamp) = val.trim().parse() {
                                if let LocalResult::Single(datetime) = Utc.timestamp_opt(timestamp, 0) {
                                    match_info.scheduled_time = Some(datetime);
                                }
                            }
                        }
                    }

                    matches_info.push(match _type {
                        InFuture => MatchBrief::InFuture(match_info),
                        Live => MatchBrief::Live(match_info),
                        Completed => MatchBrief::Completed(match_info)
                    });
                }
            }
        }

        Ok(matches_info)
    }

    /// Helper to get the HTML of the given URL.
    ///
    /// Panics if the given `url` cannot be parsed.
    #[inline]
    fn get_string<U: IntoUrl>(&self, url: U) -> Result<String> {
        let mut resp = self.client.get(url).unwrap().send()?;
        let status = resp.status();

        if !status.is_success() {
            bail!(ErrorKind::NonSuccessStatus(status));
        }

        let mut content = String::new();
        resp.read_to_string(&mut content)?;

        Ok(content)
    }
}

#[cfg(test)]
#[cfg(not(feature = "test-local-data"))]
mod test {
    use super::*;

    #[test]
    fn completed_matches_brief() {
        let scraper = Scraper::new().unwrap();
        let matches = scraper.matches_brief(Completed).unwrap();

        for _match in matches {
            // Make sure we got distinct event info
            assert!(_match.event().name != "");
            assert!(_match.event().series != "");
            assert!(_match.event().name != _match.event().series);

            // Make sure we got a value for maps won
            assert!(!_match.teams()[0].maps_won.is_none());
            assert!(!_match.teams()[1].maps_won.is_none());

            // Make sure we got distinct team names
            assert!(_match.teams()[0].name != "");
            assert!(_match.teams()[1].name != "");
            assert!(_match.teams()[0].name != _match.teams()[1].name);

            // Make sure that the methods to determine winner / loser work correctly
            assert!(_match.winner().unwrap() != _match.loser().unwrap());
            assert!(_match.winner().unwrap().maps_won > _match.loser().unwrap().maps_won);
        }
    }

    #[test]
    fn future_matches_brief() {
        let scraper = Scraper::new().unwrap();
        let matches = scraper.matches_brief(InFuture).unwrap();

        for _match in matches {
            // Make sure we got distinct event info
            assert!(_match.event().name != "");
            assert!(_match.event().series != "");
            assert!(_match.event().name != _match.event().series);

            // Make sure we didn't get a value for maps won
            assert!(_match.teams()[0].maps_won.is_none());
            assert!(_match.teams()[1].maps_won.is_none());

            // Make sure we got distinct team names
            assert!(_match.teams()[0].name != "");
            assert!(_match.teams()[1].name != "");
            assert!(_match.teams()[0].name != _match.teams()[1].name);

            // Make sure that the methods to determine winner / loser work correctly
            assert!(_match.winner().is_none());
            assert!(_match.loser().is_none());
        }
    }

    #[test]
    fn live_matches_brief() {
        let scraper = Scraper::new().unwrap();
        let matches = scraper.matches_brief(Live).unwrap();

        for _match in matches {
            // Make sure we got distinct event info
            assert!(_match.event().name != "");
            assert!(_match.event().series != "");
            assert!(_match.event().name != _match.event().series);

            // Make sure we got a value for maps won
            assert!(!_match.teams()[0].maps_won.is_none());
            assert!(!_match.teams()[1].maps_won.is_none());

            // Make sure we got distinct team names
            assert!(_match.teams()[0].name != "");
            assert!(_match.teams()[1].name != "");
            assert!(_match.teams()[0].name != _match.teams()[1].name);

            // Make sure that the methods to determine winner / loser work correctly
            assert!(_match.winner().is_none());
            assert!(_match.loser().is_none());
        }
    }
}

#[cfg(test)]
#[cfg(feature = "test-local-data")]
mod test_local_data {
    use super::*;
    use test_utils::*;

    const COMPLETED_MATCHES_BRIEF_PATH: &'static str = "test_data/completed_matches_brief.json";
    const FUTURE_MATCHES_BRIEF_PATH: &'static str = "test_data/future_matches_brief.json";
    const LIVE_MATCHES_BRIEF_PATH: &'static str = "test_data/live_matches_brief.json";

    #[test]
    fn completed_matches_brief() {
        let scraper = Scraper::new().unwrap();
        let matches = scraper.matches_brief(Completed).unwrap();

        let data = write_matches(COMPLETED_MATCHES_BRIEF_PATH, matches).unwrap();
        let loaded_data = SaveData::load(COMPLETED_MATCHES_BRIEF_PATH).unwrap();

        assert_eq!(&data, &loaded_data);
    }

    #[test]
    fn future_matches_brief() {
        let scraper = Scraper::new().unwrap();
        let matches = scraper.matches_brief(InFuture).unwrap();

        let data = write_matches(FUTURE_MATCHES_BRIEF_PATH, matches).unwrap();
        let loaded_data = SaveData::load(FUTURE_MATCHES_BRIEF_PATH).unwrap();

        assert_eq!(&data, &loaded_data);
    }

    #[test]
    fn live_matches_brief() {
        let scraper = Scraper::new().unwrap();
        let matches = scraper.matches_brief(Live).unwrap();

        let data = write_matches(LIVE_MATCHES_BRIEF_PATH, matches).unwrap();
        let loaded_data = SaveData::load(LIVE_MATCHES_BRIEF_PATH).unwrap();

        assert_eq!(&data, &loaded_data);
    }
}
