extern crate clap;
extern crate reqwest;
extern crate serde;


#[macro_use]
extern crate serde_derive;

// use clap::{Arg, App, SubCommand};
use std::io;
use anyhow::Result;
use chrono::{Utc, Duration};

use simple_logger::{SimpleLogger};

// TODO: replace logging library with macro for verbose outputting

// Limit number of days to look back for new file
const MAX_DL_DATE_LOOKBACKS: i64 = 3;

// this will be refactored so the CSV is downloaded on push to git and the path is derived
// from config file in same folder
// this will learn properties file setup
// TODO: add functionality to capture this file not existing
const BASE_CSV: &str = "/home/vscode/.config/bgg/2021-10-01.csv";

// TODO: old file purge

#[derive(Debug,Deserialize,Serialize)]
struct BoardGame {
    #[serde(rename = "ID")]
    id: u32,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Year")]
    year: u16,
    #[serde(rename = "Rank")]
    rank: u32,
    #[serde(rename = "Average")]
    average_score: f32,
    #[serde(rename = "Bayes average")]
    bayes_average: f32,
    #[serde(rename = "Users rated")]
    users_rated: u32,
    #[serde(rename = "URL")]
    bgg_url: String,
    #[serde(rename = "Thumbnail")]
    thumbnail_url: String,
}


fn boardgames_from_reader<T: io::Read>(mut rdr: csv::Reader<T>) -> Result<Vec<BoardGame>> {
    let mut boardgames = Vec::<BoardGame>::new();

    for result in rdr.deserialize() {
        let boardgame: BoardGame = result?;
        boardgames.push(boardgame);
    }

    Ok(boardgames)
}

#[allow(clippy::needless_question_mark)]
fn download_csv (date_string: &str) -> Result<Vec<BoardGame>> {
    let url = format!("https://gitlab.com/recommend.games/bgg-ranking-historicals/-/raw/master/{}.csv", &date_string);
    log::info!("Attempting download: {}", &url);
    let resp = reqwest::blocking::get(url)?.text()?;
    
    Ok(boardgames_from_reader(csv::Reader::from_reader(resp.as_bytes()))?)
}

#[allow(clippy::needless_question_mark)]
fn read_file(file_name: &str) -> Result<Vec<BoardGame>> {
    let csv_file = std::fs::File::open(file_name)?;
    
    Ok(boardgames_from_reader(csv::Reader::from_reader(csv_file))?)
}

fn save_csv(boardgames: &[BoardGame], file_path: std::path::PathBuf) -> Result<bool> {
    let mut wtr =csv::Writer::from_path(file_path)?;

    for boardgame in boardgames {
        wtr.serialize(boardgame)?;
    }
    wtr.flush()?;
    Ok(true)
}

fn load_data() -> Result<Vec<BoardGame>> {
        // Initialise UTC logger to obviate local time issue
       SimpleLogger::new()
        .with_utc_timestamps()
        .with_level(log::LevelFilter::Info)
        .init().unwrap();
    
    // Get config file path
    let config_path = match dirs::config_dir() {
        Some(config) => config.join("bgg"),
        None => panic!("User has no home directory set; home directory configuration is expected"),
    };

    // Create path for bgg in config_path
    std::fs::create_dir_all(&config_path)?;
    log::info!("Config path created: {:?}", config_path);

    let today_date = Utc::now();
    
    // assumes that a file will exist by default (accommodate where it's been deleted manually)
    let mut recent_file: Option<String> = None;

    for i in 0..MAX_DL_DATE_LOOKBACKS {
        let file_date = today_date - Duration::days(i);
        log::info!("iteration: {} date: {}", i, file_date.format("%Y-%m-%d"));
        let file_path = config_path.join(file_date.format("%Y-%m-%d.csv").to_string());

        log::info!("Looking for file: {:?}", &file_path);
        if std::path::Path::new(&file_path).exists() {
            log::info!("Recent file found: {:?}", &file_path);
            recent_file = Some(file_path.into_os_string().into_string().unwrap());
            break;
        }
    }
    

    let mut boardgames= Vec::<BoardGame>::new();
    match recent_file {
        Some(file_path) => {
            println!("Processing: {:?}", file_path);
            if let Ok(bgs) = read_file(file_path.as_str()) {
                boardgames = bgs;
            }
        },
        None => {
            let date_string = &today_date.format("%Y-%m-%d").to_string();
            if let Ok(bgs) = download_csv(date_string) {
                boardgames = bgs;

                save_csv(&boardgames, config_path.join(format!("{}.csv", date_string)))?;

            } else {
                log::info!("Using old copy of file: {}", BASE_CSV);
                boardgames = read_file(BASE_CSV)?;
            };
        },
    }

    // println!("{:?}", boardgames);

    Ok(boardgames) 
}

#[derive(Copy, Clone, Debug)]
struct RankToGame<'a> {
    rank: u32,
    boardgame: &'a BoardGame,
}

// struct PrefixToGames {
//     prefix: String
// }

fn main() -> Result<()>  {

    let mut autocomp = std::collections::HashMap::<String, Vec<RankToGame>>::new();
    let boardgame_data = &load_data()?;
    for boardgame in boardgame_data {

        // dbg!(boardgame);
        
        // create prefix
        let mut prefix=String::new();
        // tokenise name, iterate - wait on this
        for char in boardgame.name.chars() {
            let current_rank_to_game = RankToGame {
                                                    rank: boardgame.rank,
                                                    boardgame: &boardgame
                                                };
            let current_key = prefix.clone();
            prefix.push(char);
                                   
            autocomp.entry(current_key).or_insert(Vec::new()).push(current_rank_to_game);
        }
    }
    dbg!(autocomp.get("The"));

    Ok(())

}

