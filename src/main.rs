extern crate clap;
extern crate reqwest;

// use clap::{Arg, App, SubCommand};
// use reqwest::Client;
use std::{io::{copy, self, Write}, fmt::Display};
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



fn download_csv (date_string: &str) -> Result<String, reqwest::Error> {
    let url = format!("https://gitlab.com/recommend.games/bgg-ranking-historicals/-/raw/master/{}.csv", &date_string);
    log::info!("Attempting download: {}", &url);
    let resp = reqwest::blocking::get(url)?;
    
    resp.text()
    // println!("{:?}", content);
    // content


}

fn main() -> Result<()>  {
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


    let mut today_date = Utc::now();
    
    
    // assumes that a file will exist by default (accommodate where it's been deleted manually)
    let mut recent_file: Option<String> = None;

    for i in 0..MAX_DL_DATE_LOOKBACKS-1 {
        // let file_date_string = file_date.format("%Y-%m-%d");
        let file_date = today_date - Duration::days(i);
        log::info!("iteration: {} date: {}", i, file_date.format("%Y-%m-%d"));
        let file_path = config_path.join(file_date.format("%Y-%m-%d.csv").to_string());

        log::info!("Looking for file: {:?}", &file_path);
        if std::path::Path::new(&file_path).exists() {
            log::info!("Recent file found: {:?}", &file_path);
            recent_file = Some(format!("{:?}", file_path));
            break;
        }
    }
    
//https://stackoverflow.com/questions/51141672/how-do-i-ignore-an-error-returned-from-a-rust-function-and-proceed-regardless
    let mut csv_data: String;
   
    match recent_file {
        Some(file_path) => {
            csv_data = read_file(file_path.as_str())?;
        },
        None => {
            let date_string = &today_date.format("%Y-%m-%d").to_string();
            if let Ok(csv_string) = download_csv(date_string) {
                csv_data = csv_string;
                // log::info!("STILL GOT DATE STRING: {:?}",format!("{}.csv", date_string));
                let output_file = config_path.join(format!("{}.csv", date_string));
                let mut out = std::fs::File::create(output_file)?;
                // io::copy(&mut csv_data, &mut out)?;
                write!(out, "{}", csv_data);


            } else {
                log::info!("Using old copy of file: {}", BASE_CSV);
                csv_data = read_file(BASE_CSV)?;
            };
        },
    }
    


    Ok(())
}

fn read_file(file_name: &str) -> Result<String> {
    if let Ok(csv_string) = std::fs::read_to_string(file_name) {
        Ok(csv_string)
    } else {
        panic!("Unable to read: {}", file_name);
    }
}