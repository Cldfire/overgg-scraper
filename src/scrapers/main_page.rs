//! Handles extraction of content from the main page (https://www.over.gg/).

use error::*;
use super::load_sels;
use scraper::Html;
use chrono::{Utc, TimeZone, LocalResult};
use data_structs::{MatchBrief, MatchBriefInfo, MatchBriefType, LivestreamInfo};
use data_structs::MatchBriefType::*;

const MATCHES_BRIEF_SELECTORS_STR: &'static str = include_str!("../../selectors/matches_brief.toml");
const LIVE_STREAMS_SELECTORS_STR: &'static str = include_str!("../../selectors/live_streams.toml");

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
        let selectors = load_sels(MATCHES_BRIEF_SELECTORS_STR);

        // First we get the lists of upcoming / completed matches
        for list in self.doc.select(&selectors["matches"]) {
            let header_text = match list.select(&selectors["header"]).next() {
                Some(elem) => elem.text().collect::<String>(),
                // TODO: Don't bail if no header
                None => bail!(ErrorKind::ExtractionError)
            };
            
            // Now we narrow down to the list that contains the match type we want
            if header_text.trim() == String::from(_type) {
                // Now we get the match type we want
                let matches: Vec<::scraper::ElementRef> = 
                    list.select(&selectors["match"]).filter(|e| {
                        // If we want live matches, we need to do some filtering
                        if _type == Live {
                            match e.select(&selectors["live"]).next() {
                                Some(_) => true,
                                None => false
                            }
                        // If we want future matches, we need to do filtering too
                        } else if _type == InFuture {
                            match e.select(&selectors["live"]).next() {
                                Some(_) => false,
                                None => true
                            }
                        // No filtering is required for completed matches
                        } else if _type == Completed {
                            true
                        } else {
                            unimplemented!();
                        }
                    }).collect();

                // Finally we get information for each match
                for _match in matches {
                    let mut match_info = MatchBriefInfo::default();

                    // Event name
                    if let Some(elem) = _match.select(&selectors["event_name"]).next() {
                        match_info.event.name = elem.text().collect::<String>().trim().into();
                    }

                    // Event series
                    if let Some(elem) = _match.select(&selectors["event_series"]).next() {
                        match_info.event.series = elem.text().collect::<String>().trim().into();
                    }

                    let teams_sel = &selectors["teams"];
                    let mut teams = _match.select(teams_sel);

                    for i in 0..2 {
                        if let Some(team) = teams.next() {
                            // Team name
                            if let Some(elem) = team.select(&selectors["team_name"]).next() {
                                match_info.teams[i].name = elem.text().collect::<String>().trim().into();
                            }

                            // Team won maps count
                            if let Some(elem) = team.select(&selectors["team_score"]).next() {
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
                    if let Some(elem) = _match.select(&selectors["match_scheduled_time"]).next() {
                        // TODO: Get attr keys in file
                        if let Some(val) = elem.value().attr("data-utc-ts") {
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

                break;
            }
        }

        Ok(matches_info)
    }

    /// Gets the livestream information available on the main page.
    ///
    /// This differs from simply grabbing OW livestreams from Twitch in that it
    /// is a curated list consisting of pro players and tournament / event streams.
    #[inline]
    pub fn live_streams(&self) -> Result<Vec<LivestreamInfo>> {
        let mut live_streams = vec![];
        let selectors = load_sels(LIVE_STREAMS_SELECTORS_STR);

        // First we have to find the card that contains the livestream information
        for card in self.doc.select(&selectors["cards"]) {

            if let Some(elem) = card.select(&selectors["header"]).next() {
                if elem.text().collect::<String>().trim() == "Live Streams" {
                    // Now we extract the info for each stream
                    for stream in card.select(&selectors["stream"]) {
                        let mut stream_info = LivestreamInfo::default();

                        // Stream name
                        if let Some(elem) = stream.select(&selectors["stream_name"]).next() {
                            stream_info.name = elem.text().collect::<String>().trim().into();
                        }

                        // Stream title
                        if let Some(val) = stream.value().attr("title") {
                            stream_info.title = Some(val.trim().into());
                        }

                        // Stream viewer count
                        if let Some(elem) = stream.select(&selectors["stream_viewer_count"]).next() {
                            stream_info.viewer_count = match elem.text()
                                                                .collect::<String>()
                                                                .trim()
                                                                .parse() {
                                Ok(val) => Some(val),
                                Err(_) => None
                            }
                        }

                        // Stream URL
                        if let Some(val) = stream.value().attr("href") {
                            stream_info.url = val.trim().into();
                        }

                        live_streams.push(stream_info);
                    }

                    break;
                }
            }
        }

        Ok(live_streams)
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
            if let (Some(winner), Some(loser)) = (_match.winner(), _match.loser()) {
                assert!(winner != loser);
                assert!(winner.maps_won > loser.maps_won);
            }
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
        }
    }

    #[test]
    fn live_streams() {
        let dl = Downloader::new().unwrap();
        let scraper = dl.main_page().unwrap();
        let streams = scraper.live_streams().unwrap();

        for stream in streams {
            // Make sure we got a name
            assert!(stream.name != "");
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

    // TODO: live_streams()
}
