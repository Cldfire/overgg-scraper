//! Handles extraction of content from the main page (https://www.over.gg/).

use error::*;
use super::sel;
use scraper::Html;
use chrono::{Utc, TimeZone, LocalResult};
use data_structs::{MatchBrief, MatchBriefInfo, MatchBriefType};
use data_structs::MatchBriefType::*;
use std::collections::HashMap;

const MATCHES_BRIEF_SELECTORS_STR: &'static str = include_str!("../../selectors/matches_brief.toml");

/// Handles extraction of content from the main page (https://www.over.gg/).
///
/// You can either provide your own HTML string (see the `From<String>` impl)
/// or use the `http_client::Downloader`.
pub struct MainPageScraper {
    doc: Html
}

impl From<String> for MainPageScraper {
    fn from(html: String) -> Self {
        MainPageScraper {
            doc: Html::parse_document(&html)
        }
    }
}

impl MainPageScraper {
    /// Gets information available on the main page for the matches of the given
    /// type.
    #[inline]
    pub fn matches_brief(&self, _type: MatchBriefType) -> Result<Vec<MatchBrief>> {
        let mut matches_info = vec![];

        let selectors: HashMap<String, String> =
            ::toml::from_str(MATCHES_BRIEF_SELECTORS_STR).unwrap();

        // First we get the lists of upcoming / completed matches
        for list in self.doc.select(&(sel(&selectors["matches"]))) {
            let header_text = match list.select(&(sel(&selectors["header"]))).next() {
                Some(elem) => elem.text().collect::<String>(),
                None => bail!(ErrorKind::ExtractionError)
            };
            
            // Now we narrow down to the list that contains the match type we want
            if header_text.trim() == String::from(_type) {
                // Now we get the match type we want
                let matches: Vec<::scraper::ElementRef> = 
                    list.select(&(sel(&selectors["match"]))).filter(|e| {
                        // If we want live matches, we need to do some filtering
                        if _type == Live {
                            match e.select(&(sel(&selectors["live"]))).next() {
                                Some(_) => true,
                                None => false
                            }
                        // If we want future matches, we need to do filtering too
                        } else if _type == InFuture {
                            match e.select(&(sel(&selectors["live"]))).next() {
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
                    if let Some(elem) = _match.select(&(sel(&selectors["event_name"]))).next() {
                        match_info.event.name = elem.text().collect::<String>().trim().into();
                    }

                    // Event series
                    if let Some(elem) = _match.select(&(sel(&selectors["event_series"]))).next() {
                        match_info.event.series = elem.text().collect::<String>().trim().into();
                    }

                    let teams_sel = &(sel(&selectors["teams"]));
                    let mut teams = _match.select(teams_sel);

                    for i in 0..2 {
                        if let Some(team) = teams.next() {
                            // Team name
                            if let Some(elem) = team.select(&(sel(&selectors["team_name"]))).next() {
                                match_info.teams[i].name = elem.text().collect::<String>().trim().into();
                            }

                            // Team won maps count
                            if let Some(elem) = team.select(&(sel(&selectors["team_score"]))).next() {
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
                    if let Some(elem) = _match.select(&(sel(&selectors["match_scheduled_time"]))).next() {
                        if let Some(val) = elem.value().attr(&selectors["timestamp_attr"]) {
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
}

#[cfg(test)]
#[cfg(not(feature = "test-local-data"))]
mod test {
    use super::*;
    use http_client::Downloader;

    #[test]
    fn completed_matches_brief() {
        let dl = Downloader::new().unwrap();
        let scraper = dl.main_page().unwrap();
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
        let dl = Downloader::new().unwrap();
        let scraper = dl.main_page().unwrap();
        let matches = scraper.matches_brief(InFuture).unwrap();

        for _match in matches {
            // Make sure we got distinct event info
            assert!(_match.event().name != "");
            assert!(_match.event().series != "");
            assert!(_match.event().name != _match.event().series);

            // Make sure we didn't get a value for maps won
            assert!(_match.teams()[0].maps_won.is_none());
            assert!(_match.teams()[1].maps_won.is_none());

            // Make sure we got non-empty team names
            // Distinctness is not always possible for upcoming matches as the
            // teams could be listed as TBD
            assert!(_match.teams()[0].name != "");
            assert!(_match.teams()[1].name != "");

            // Make sure that the methods to determine winner / loser work correctly
            assert!(_match.winner().is_none());
            assert!(_match.loser().is_none());
        }
    }

    #[test]
    fn live_matches_brief() {
        let dl = Downloader::new().unwrap();
        let scraper = dl.main_page().unwrap();
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

    const TEST_DATA_MAIN_PAGE: &'static str = include_str!("../../test_data/www.over.gg.html");
    const COMPLETED_MATCHES_BRIEF_PATH: &'static str = "test_data/completed_matches_brief.json";
    const FUTURE_MATCHES_BRIEF_PATH: &'static str = "test_data/future_matches_brief.json";
    const LIVE_MATCHES_BRIEF_PATH: &'static str = "test_data/live_matches_brief.json";

    #[test]
    fn completed_matches_brief() {
        let scraper = MainPageScraper::from(String::from(TEST_DATA_MAIN_PAGE));
        let matches = scraper.matches_brief(Completed).unwrap();

        let data = write_matches(COMPLETED_MATCHES_BRIEF_PATH, matches).unwrap();
        let loaded_data = SaveData::load(COMPLETED_MATCHES_BRIEF_PATH).unwrap();

        assert_eq!(&data, &loaded_data);
    }

    #[test]
    fn future_matches_brief() {
        let scraper = MainPageScraper::from(String::from(TEST_DATA_MAIN_PAGE));
        let matches = scraper.matches_brief(InFuture).unwrap();

        let data = write_matches(FUTURE_MATCHES_BRIEF_PATH, matches).unwrap();
        let loaded_data = SaveData::load(FUTURE_MATCHES_BRIEF_PATH).unwrap();

        assert_eq!(&data, &loaded_data);
    }

    #[test]
    fn live_matches_brief() {
        let scraper = MainPageScraper::from(String::from(TEST_DATA_MAIN_PAGE));
        let matches = scraper.matches_brief(Live).unwrap();

        let data = write_matches(LIVE_MATCHES_BRIEF_PATH, matches).unwrap();
        let loaded_data = SaveData::load(LIVE_MATCHES_BRIEF_PATH).unwrap();

        assert_eq!(&data, &loaded_data);
    }
}