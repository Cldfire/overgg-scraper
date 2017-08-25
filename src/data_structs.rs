#![allow(unused)]

use chrono::{DateTime, Utc};

pub enum MatchType {
    Bo3,
    Bo5
}

/// Used to specify whether completed or future matches should be extracted.
#[derive(Debug, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MatchBriefType {
    InFuture,
    Live,
    Completed
}

impl From<MatchBriefType> for String {
    fn from(_type: MatchBriefType) -> String {
        use MatchBriefType::*;

        match _type {
            Completed => "completed matches".into(),
            InFuture | Live => "upcoming matches".into()
        }
    }
}

/// Used to statically type the two teams in a match.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Team {
    Zero,
    One
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MatchBrief {
    /// A match that has not been played yet.
    InFuture(MatchBriefInfo),

    /// A match that is in progress.
    Live(MatchBriefInfo),

    /// A match that has been played.
    Completed(MatchBriefInfo)
}

impl MatchBrief {
    /// Determines which team won the match.
    ///
    /// Will be `None` if neither team won (match could be a draw or just not finished).
    #[inline]
    pub fn winner(&self) -> Option<&TeamCompletedMatchBriefInfo> {
        use MatchBrief::*;

        match self {
            &Completed(ref info) => {
                info.winner()
            }
            &InFuture(_) |
            &Live(_) => None
        }
    }

    pub fn set_team_name(&mut self, team: Team, val: String) {
        use MatchBrief::*;
        use self::Team::*;

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

    pub fn set_team_maps_won(&mut self, team: Team, val: u8) {
        use MatchBrief::*;
        use self::Team::*;

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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
}

#[derive(Default, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TeamCompletedMatchBriefInfo {
    pub name: String,
    pub maps_won: Option<u8>
}

pub struct CompletedMatch {
    pub match_type: MatchType,
    pub event: EventInfo,
    pub scheduled_time: Option<DateTime<Utc>>,
    // TODO: Streams
    pub team_zero: TeamCompletedMatchInfo,
    pub team_one: TeamCompletedMatchInfo
    // TODO: Comments
    // TODO: Map vods
}

impl CompletedMatch {
    /// Determines which team won the match.
    ///
    /// Will be `None` if neither team won (match was a draw).
    #[inline]
    pub fn winner(&self) -> Option<&TeamCompletedMatchInfo> {
        if self.team_zero.maps_won > self.team_one.maps_won {
            Some(&(self.team_zero))
        } else if self.team_zero.maps_won < self.team_one.maps_won {
            Some(&(self.team_one))
        } else {
            None
        }
    }
}

pub struct TeamCompletedMatchInfo {
    pub name: String,
    pub ranking: i32,
    pub maps_won: u8,
    // TODO: Map info
}

#[derive(Default, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EventInfo {
    pub name: String,
    pub series: String
}
