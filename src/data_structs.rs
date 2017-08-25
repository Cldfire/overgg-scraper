#![allow(unused)]

use chrono::{DateTime, Utc};

pub enum MatchType {
    Bo3,
    Bo5
}

/// Used to specify whether completed or future matches should be extracted.
#[derive(Debug, PartialEq, Copy, Clone)]
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
pub enum Team {
    Zero,
    One
}

#[derive(Debug)]
pub enum MatchBrief {
    /// A match that has not been played yet.
    InFuture {
        event: EventInfo,
        team_names: [String; 2],
        scheduled_time: Option<DateTime<Utc>>
    },

    Live {
        // TODO: Live matches
    },

    /// A match that has been played.
    Completed {
        event: EventInfo,
        teams: [TeamCompletedMatchBriefInfo; 2],
        scheduled_time: Option<DateTime<Utc>>
    }
}

impl MatchBrief {
    pub fn set_team_name(&mut self, team: Team, val: String) {
        use MatchBrief::*;
        use self::Team::*;

        match self {
            &mut InFuture { ref event, ref mut team_names, ref scheduled_time } => {
                match team {
                    Zero => team_names[0] = val,
                    One => team_names[1] = val
                }
            }

            &mut Live {} => {} // TODO: Live

            &mut Completed { ref event, ref mut teams, ref scheduled_time } => {
                match team {
                    Zero => teams[0].name = val,
                    One => teams[1].name = val
                }
            }
        }
    }

    pub fn set_team_maps_won(&mut self, team: Team, val: u8) {
        use MatchBrief::*;
        use self::Team::*;

        match self {
            &mut InFuture { ref event, ref team_names, ref scheduled_time } => {}

            &mut Live {} => {} // TODO: Live

            &mut Completed { ref event, ref mut teams, ref scheduled_time } => {
                match team {
                    Zero => teams[0].maps_won = val,
                    One => teams[1].maps_won = val
                }
            }
        }
    }
}

#[derive(Default, Debug)]
pub struct TeamCompletedMatchBriefInfo {
    pub name: String,
    pub maps_won: u8
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

#[derive(Default, Debug)]
pub struct EventInfo {
    pub name: String,
    pub series: String
}
