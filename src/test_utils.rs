use crate::data_structs::MatchBriefInfo;
use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use std::io::{Read, Write};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json;
use crate::error::*;

#[derive(Debug, PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct SaveData {
    pub match_data: Vec<MatchBriefInfo>,
    pub winners: Vec<Option<String>>
}

impl SaveData {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        load(path)
    }
}

pub fn write_matches<P: AsRef<Path>>(path: P, matches: Vec<MatchBriefInfo>) -> Result<SaveData> {
    let winners = matches.iter().map(|t| t.winner().map(|w| w.name.clone())).collect();

    let data = SaveData {
        match_data: matches,
        winners
    };

    write_non_overwrite(path, &data)?;
    Ok(data)
}

pub fn load<P, S>(path: P) -> Result<S> 
    where P: AsRef<Path>,
          S: DeserializeOwned {

    let file = File::open(path)?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;

    Ok(serde_json::from_str(&contents)?)
}

pub fn write_non_overwrite<P, S>(path: P, thing: &S) -> Result<()>
    where P: AsRef<Path>,
          S: Serialize {
            
    let path = path.as_ref();

    if !path.exists() {
        let mut file = File::create(path)?;
        file.write_all(serde_json::to_string(thing)?.as_bytes())?;
    }

    Ok(())
}
