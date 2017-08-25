#[macro_use]
extern crate error_chain;
extern crate reqwest;
extern crate scraper;
extern crate chrono;

mod error;
mod data_structs;

use error::*;
use reqwest::{Client, IntoUrl};
use scraper::{Html, Selector};
use chrono::{Utc, TimeZone, LocalResult};
use data_structs::{EventInfo, MatchBrief, MatchBriefType, TeamCompletedMatchBriefInfo};
use data_structs::MatchBriefType::*;
use std::io::Read;

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
        let html = self.get_string("https://www.over.gg/")?;
        let doc = Html::parse_document(&html);
        let mut matches = vec![];

        // TODO: Get this out of the code and into a config file
        let completed_matches_sel = 
            Selector::parse("div.wf-module.wf-card.mod-home-matches").unwrap();
        let header_sel = Selector::parse("div.wf-module-header").unwrap();
        let match_sel = Selector::parse("a.wf-module-item.mod-match").unwrap();
        let event_name_sel = Selector::parse("div.h-match-preview-event").unwrap();
        let event_series_sel = Selector::parse("div.h-match-preview-series").unwrap();
        let teams_sel = Selector::parse("div.h-match-team").unwrap();
        let team_name_sel = Selector::parse("div.h-match-team-name").unwrap();
        let team_score_sel = Selector::parse("div.h-match-team-score.mod-count").unwrap();
        let match_scheduled_time_sel = Selector::parse("div.h-match-preview-time").unwrap();
        let timestamp_attr_name = "data-utc-ts";

        // First we get the list of completed matches
        for list in doc.select(&completed_matches_sel) {
            let header_text = match list.select(&header_sel).next() {
                Some(elem) => elem.text().collect::<String>(),
                None => bail!(ErrorKind::ExtractionError(html))
            };
            
            if header_text.trim() == String::from(_type) {
                // Now we get information for each match
                for _match in list.select(&match_sel) {
                    let mut event = EventInfo::default();
                    let mut teams_info = [
                        TeamCompletedMatchBriefInfo::default(),
                        TeamCompletedMatchBriefInfo::default()
                    ];
                    let mut scheduled_time = None;

                    // Event name
                    if let Some(elem) = _match.select(&event_name_sel).next() {
                        event.name = elem.text().collect::<String>().trim().into();
                    }

                    // Event series
                    if let Some(elem) = _match.select(&event_series_sel).next() {
                        event.series = elem.text().collect::<String>().trim().into();
                    }

                    let mut teams = _match.select(&teams_sel);

                    for i in 0..2 {
                        if let Some(team) = teams.next() {
                            // Team name
                            if let Some(elem) = team.select(&team_name_sel).next() {
                                teams_info[i].name = elem.text().collect::<String>().trim().into();
                            }

                            // Team won maps count
                            if _type == Completed {
                                if let Some(elem) = team.select(&team_score_sel).next() {
                                    // TODO: Is just setting this to 0 when it fails to parse acceptable
                                    teams_info[i].maps_won = elem.text()
                                                                .collect::<String>()
                                                                .trim()
                                                                .parse()
                                                                .unwrap_or(0);
                                }
                            }
                        }
                    }

                    // Scheduled match time
                    if let Some(elem) = _match.select(&match_scheduled_time_sel).next() {
                        if let Some(val) = elem.value().attr(timestamp_attr_name) {
                            if let Ok(timestamp) = val.trim().parse() {
                                if let LocalResult::Single(datetime) = Utc.timestamp_opt(timestamp, 0) {
                                    scheduled_time = Some(datetime);
                                }
                            }
                        }
                    }

                    matches.push(match _type {
                        InFuture => {
                            MatchBrief::InFuture {
                                event,
                                team_names: [
                                    // TODO: Why do I have to clone here
                                    teams_info[0].name.clone(),
                                    teams_info[1].name.clone()
                                ],
                                scheduled_time
                            }
                        },

                        Live => {
                            MatchBrief::Live{} // TODO: Live
                        },

                        Completed => {
                            MatchBrief::Completed {
                                event,
                                teams: teams_info,
                                scheduled_time
                            }
                        },
                    });
                }
            }
        }

        Ok(matches)
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
mod test {
    use super::*;

    #[test]
    fn completed_matches_brief() {
        let scraper = Scraper::new().unwrap();
        let matches = scraper.matches_brief(Completed).unwrap();

        println!("{:#?}", matches);
    }

    #[test]
    fn future_matches_brief() {
        let scraper = Scraper::new().unwrap();
        let matches = scraper.matches_brief(InFuture).unwrap();

        println!("{:#?}", matches);
    }
}
