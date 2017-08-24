#[macro_use]
extern crate error_chain;
extern crate reqwest;
extern crate scraper;

mod error;
mod data_structs;

use error::*;
use reqwest::{Client, IntoUrl};
use scraper::{Html, Selector};
use data_structs::CompletedMatchBrief;
use std::io::Read;

pub struct Scraper {
    client: Client
}

impl Scraper {
    /// Get a scraper with reqwest's default `Client` config.
    pub fn new() -> Result<Scraper> {
        Ok(Scraper {
            client: Client::new()?
        })
    }

    /// Get a scraper that will use the `Client` you provide.
    ///
    /// This means you can configure the client as you wish.
    pub fn with_client(client: Client) -> Scraper {
        Scraper {
            client
        }
    }

    /// Gets the main page (https://www.over.gg/) and extracts the completed
    /// match info available there for all listed matches.
    pub fn matches_brief(&self) -> Result<Vec<CompletedMatchBrief>> {
        let html = self.get_string("https://www.over.gg/")?;
        let doc = Html::parse_document(&html);
        let mut matches = vec![];

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

        // First we get the list of completed matches
        for list in doc.select(&completed_matches_sel) {
            let header_text = match list.select(&header_sel).next() {
                Some(elem) => elem.text().collect::<String>(),
                None => bail!(ErrorKind::ExtractionError(html))
            };
            
            if header_text.trim() == "completed matches" {
                // Now we get information for each match
                for _match in list.select(&match_sel) {
                    let mut match_info = CompletedMatchBrief::default();

                    // Event name
                    if let Some(elem) = _match.select(&event_name_sel).next() {
                        match_info.event.name = elem.text().collect::<String>().trim().into();
                    }

                    // Event series
                    if let Some(elem) = _match.select(&event_series_sel).next() {
                        match_info.event.series = elem.text().collect::<String>().trim().into();
                    }

                    let mut teams = _match.select(&teams_sel);

                    // TODO: Remove code duplication here
                    if let Some(team) = teams.next() {
                        // First team's name
                        if let Some(elem) = team.select(&team_name_sel).next() {
                            match_info.team_zero.name = elem.text().collect::<String>().trim().into();
                        }

                        // First team's won maps count
                        if let Some(elem) = team.select(&team_score_sel).next() {
                            // TODO: Is just setting this to 0 when it fails to parse acceptable
                            match_info.team_zero.maps_won = elem.text()
                                                                .collect::<String>()
                                                                .trim()
                                                                .parse()
                                                                .unwrap_or(0);
                        }
                    }

                    if let Some(team) = teams.next() {
                        // Second team's name
                        if let Some(elem) = team.select(&team_name_sel).next() {
                            match_info.team_one.name = elem.text().collect::<String>().trim().into();
                        }

                        // Second team's won maps count
                        if let Some(elem) = team.select(&team_score_sel).next() {
                            // TODO: Is just setting this to 0 when it fails to parse acceptable
                            match_info.team_one.maps_won = elem.text()
                                                               .collect::<String>()
                                                               .trim()
                                                               .parse()
                                                               .unwrap_or(0);
                        }
                    }

                    // // Scheduled match time
                    // if let Some(elem) = _match.select(&match_scheduled_time_sel).next() {
                    //     if let Some(val) = elem.attr("data-utc-ts") {
                    //         // TODO: Is just setting this to 0 when it fails to parse acceptable
                    //         match_info.scheduled_time = val.parse().unwrap_or(0);
                    //     } 
                    // }

                    matches.push(match_info);
                }
            }
        }

        Ok(matches)
    }

    /// Helper to get the HTML of the given URL.
    ///
    /// Panics if the given `url` cannot be parsed.
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
    fn matches_brief() {
        let scraper = Scraper::new().unwrap();
        let matches = scraper.matches_brief().unwrap();

        println!("{:#?}", matches);
    }
}
