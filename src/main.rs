extern crate clap;
extern crate reqwest;

// use clap::{Arg, App, SubCommand};
// use reqwest::Client;
use std::{io::{copy, self}, fmt::Display};
use anyhow::Result;
use chrono::{Utc, Duration};
use simple_logger::{SimpleLogger};

// Limit number of days to look back for new file
const MAX_DL_DATE_LOOKBACKS: i64 = 3;

// this will be refactored so the CSV is downloaded on push to git and the path is derived
// from config file in same folder
// this will learn properties file setup
// TODO: add functionality to capture this file not existing
const BASE_CSV: &str = "/home/vscode/.config/bgg/2021-10-01.csv";

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


    let mut file_date = Utc::now();
    
    // check file existence 
    // if not found DL, on fail, fail
    // if found, check 3 days
    // if none found, DL (update), on fail, warn
    // if found load
    
    // assumes that a file will exist by default (accommodate where it's been deleted manually)


    for i in 0..MAX_DL_DATE_LOOKBACKS-1 {
        // let file_date_string = file_date.format("%Y-%m-%d");
        file_date = file_date - Duration::days(1);
        log::info!("iteration: {} date: {}", i, file_date.format("%Y-%m-%d"));
        let file_name = config_path.join(file_date.format("%Y-%m-%d.csv").to_string());

        log::info!("Looking for file: {:?}", &file_name);
        if std::path::Path::new(&file_name).exists() {
            log::info!("File exists: {:?}", &file_name);
        }
    }
    
    let _a = match download_csv(&file_date.format("%Y-%m-%d").to_string()) {
        Ok(csv_string) => csv_string,
        Err(e) => {
            log::error!("{}", &e.to_string());
            panic!("CSV not downloaded")
        },
    };

    // let resp = reqwest::blocking::get("https://gitlab.com/recommend.games/bgg-ranking-historicals/-/raw/master/2021-12-15.csv");
    // let mut content = match resp {
    //     Ok(content) => content,
    //     Err(error) => {eprintln!("{:?}", error);
    //                          }
    // };
    
    // let mut out = std::fs::File::create("2021-12-16.csv")?;
    // io::copy(&mut resp, &mut out)?;


    Ok(())
}