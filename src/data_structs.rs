#![allow(unused)]

use chrono::{DateTime, Utc};

pub enum MatchType {
    Bo3,
    Bo5
}

#[derive(Default, Debug)]
pub struct CompletedMatchBrief {
    pub event: EventInfo,
    pub team_zero: TeamCompletedMatchBriefInfo,
    pub team_one: TeamCompletedMatchBriefInfo,
    pub scheduled_time: Option<DateTime<Utc>>
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
