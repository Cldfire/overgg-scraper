#![allow(unused)]

use chrono::{DateTime, Utc};
use self::MatchBrief::*;
use self::MatchBriefType::InFuture as InFutureType;
use self::MatchBriefType::Live as LiveType;
use self::MatchBriefType::Completed as CompletedType;
use self::Team::*;

pub enum MatchType {
    Bo3,
    Bo5
}

/// Used to specify whether completed or future matches should be extracted.
#[derive(Debug, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "derive-serde", derive(Serialize, Deserialize))]
pub enum MatchBriefType {
    InFuture,
    Live,
    Completed
}

impl From<MatchBriefType> for String {
    fn from(_type: MatchBriefType) -> String {
        match _type {
            CompletedType => "completed matches".into(),
            InFutureType | LiveType => "upcoming matches".into()
        }
    }
}

/// Used to statically type the two teams in a match.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "derive-serde", derive(Serialize, Deserialize))]
pub enum Team {
    Zero,
    One
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "derive-serde", derive(Serialize, Deserialize))]
pub enum MatchBrief {
    /// A match that has not been played yet.
    InFuture(MatchBriefInfo),

    /// A match that is in progress.
    Live(MatchBriefInfo),

    /// A match that has been played.
    Completed(MatchBriefInfo)
}

impl MatchBrief {
    /// Getter for the `event` field.
    #[inline]
    pub fn event(&self) -> &EventInfo {
        match self {
            &InFuture(ref info) |
            &Live(ref info) |
            &Completed(ref info) => {
                &(info.event)
            }
        }
    }

    /// Getter for the `teams` field.
    #[inline]
    pub fn teams(&self) -> &[TeamCompletedMatchBriefInfo; 2] {
        match self {
            &InFuture(ref info) |
            &Live(ref info) |
            &Completed(ref info) => {
                &(info.teams)
            }
        }
    }

    /// Determines which team won the match.
    ///
    /// Will be `None` if neither team won (match could be a draw or just not finished).
    #[inline]
    pub fn winner(&self) -> Option<&TeamCompletedMatchBriefInfo> {
        match self {
            &Completed(ref info) => {
                info.winner()
            }
            &InFuture(_) |
            &Live(_) => None
        }
    }

    /// Determines which team lost the match.
    ///
    /// Will be `None` if neither team lost (match could be a draw or just not finished).
    #[inline]
    pub fn loser(&self) -> Option<&TeamCompletedMatchBriefInfo> {
        match self {
            &Completed(ref info) => {
                info.loser()
            }
            &InFuture(_) |
            &Live(_) => None
        }
    }

    pub(crate) fn set_team_name(&mut self, team: Team, val: String) {
        match self {
            &mut InFuture(ref mut info) |
            &mut Live(ref mut info) |
            &mut Completed(ref mut info) => {
                match team {
                    Zero => info.teams[0].name = val,
                    One => info.teams[1].name = val
                }
            }
        }
    }

    pub(crate) fn set_team_maps_won(&mut self, team: Team, val: u8) {
        match self {
            &mut InFuture(ref mut info) |
            &mut Live(ref mut info) |
            &mut Completed(ref mut info) => {
                match team {
                    Zero => info.teams[0].maps_won = Some(val),
                    One => info.teams[1].maps_won = Some(val)
                }
            }
        }
    }
}

#[derive(Debug, Default, PartialEq)]
#[cfg_attr(feature = "derive-serde", derive(Serialize, Deserialize))]
pub struct MatchBriefInfo {
    pub event: EventInfo,
    pub teams: [TeamCompletedMatchBriefInfo; 2],
    pub scheduled_time: Option<DateTime<Utc>>
}

impl MatchBriefInfo {
    /// Determines which team won the match.
    ///
    /// Will be `None` if neither team won (match was a draw).
    #[inline]
    pub fn winner(&self) -> Option<&TeamCompletedMatchBriefInfo> {
        if self.teams[0].maps_won > self.teams[1].maps_won {
            Some(&(self.teams[0]))
        } else if self.teams[0].maps_won < self.teams[1].maps_won {
            Some(&(self.teams[1]))
        } else {
            None
        }
    }

    /// Determines which team lost the match.
    ///
    /// Will be `None` if neither team lost (match was a draw).
    #[inline]
    pub fn loser(&self) -> Option<&TeamCompletedMatchBriefInfo> {
        if self.teams[0].maps_won < self.teams[1].maps_won {
            Some(&(self.teams[0]))
        } else if self.teams[0].maps_won > self.teams[1].maps_won {
            Some(&(self.teams[1]))
        } else {
            None
        }
    }
}

#[derive(Default, Debug, PartialEq)]
#[cfg_attr(feature = "derive-serde", derive(Serialize, Deserialize))]
pub struct TeamCompletedMatchBriefInfo {
    pub name: String,
    pub maps_won: Option<u8>
}

#[derive(Default, Debug, PartialEq)]
#[cfg_attr(feature = "derive-serde", derive(Serialize, Deserialize))]
pub struct EventInfo {
    pub name: String,
    pub series: String
}

#[derive(Default, Debug, PartialEq)]
#[cfg_attr(feature = "derive-serde", derive(Serialize, Deserialize))]
pub struct LivestreamInfo {
    pub name: String,
    pub title: Option<String>,
    pub viewer_count: Option<u32>,
    // TODO: Maybe this should be a `reqwest::Url` when `http_client` is on
    pub url: String
}

// TODO: Finish the beneath draft

pub struct CompletedMatch {
    pub match_type: MatchType,
    pub event: EventInfo,
    pub scheduled_time: Option<DateTime<Utc>>,
    // TODO: Streams
    pub teams: [TeamCompletedMatchInfo; 2]
    // TODO: Comments
    // TODO: Map vods
}

pub struct TeamCompletedMatchInfo {
    pub name: String,
    pub ranking: i32,
    pub maps_won: u8,
    // TODO: Map info
}
