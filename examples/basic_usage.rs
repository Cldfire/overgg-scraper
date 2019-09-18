extern crate overgg_scraper;

// You may want your own error setup in your own code; here we just use the scraper's
// error types.
use overgg_scraper::error::*;
use overgg_scraper::http_client::Downloader;
use overgg_scraper::data_structs::MatchBriefType::*;

fn main() {
    match actual_main() {
        Ok(()) => {},
        Err(e) => println!("\n{:?}. \nDescription: {:?}\n", e, e.description())
    }
}

// We write a function so that we can return a `Result` and use `?`
fn actual_main() -> Result<()> {
    let dlr = Downloader::new();
    let scraper = dlr.main_page()?;

    // Get the data...
    let completed_matches = scraper.matches_brief(Completed);
    let upcoming_matches = scraper.matches_brief(InFuture);
    let live_matches = scraper.matches_brief(Live);
    let mut streams = scraper.live_streams();

    // ...use the data.
    println!();

    {
        let winner = completed_matches[0].winner().unwrap();
        let loser = completed_matches[0].loser().unwrap();

        println!("Completed match #1: {} beat {} by a score of {} - {}",
            winner.name,
            loser.name,
            winner.maps_won.unwrap(),
            loser.maps_won.unwrap());
    }

    println!("Upcoming match #1: {} plays {} at {}",
        upcoming_matches[0].teams[0].name,
        upcoming_matches[0].teams[1].name,
        upcoming_matches[0].scheduled_time.unwrap());

    if let Some(_match) = live_matches.get(0) {
        println!("Live match #1: {} is playing {} and the score is currently {} - {}",
            _match.teams[0].name,
            _match.teams[1].name,
            _match.teams[0].maps_won.unwrap(),
            _match.teams[1].maps_won.unwrap());
    }

    if streams.curated.len() > 0 {
        let stream = streams.curated.remove(0);

        println!("Curated streamer #1: {} is streaming \"{}\" to {} viewers",
            stream.name,
            stream.title.unwrap(),
            stream.viewer_count.unwrap());
    }

    println!();

    Ok(())
}
