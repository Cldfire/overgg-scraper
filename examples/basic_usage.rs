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
    let dlr = Downloader::new()?;
    let scraper = dlr.main_page()?;

    let completed_matches = scraper.matches_brief(Completed)?;
    let upcoming_matches = scraper.matches_brief(InFuture)?;
    let live_matches = scraper.matches_brief(Live)?;

    println!("First completed match:\n {:#?}\n", completed_matches[0]);
    println!("First upcoming match:\n {:#?}\n", upcoming_matches[0]);
    
    match live_matches.get(0) {
        Some(_match) => println!("First live match:\n {:#?}", _match),
        None => {}
    }

    Ok(())
}
