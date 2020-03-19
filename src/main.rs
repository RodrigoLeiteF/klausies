#[macro_use]
extern crate log;
extern crate env_logger;

use chrono;
use clap::Clap;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};
use toml;
use xdg::BaseDirectories;

mod config;
mod listenbrainz;
mod mpd;

#[derive(Clap)]
#[clap(version = "1.0", author = "Rodrigo Leite <rodrigo@leite.dev>")]
struct Opts {
    /// Sets a custom config file. Could have been an Option<T> with no default too
    #[clap(short = "c", long = "config")]
    config: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    last_fm: Option<LastFMConfig>,
    listenbrainz: Option<ListenbrainzConfig>,
    mpd: Option<MpdConfig>,
}

#[derive(Deserialize, Serialize)]
struct LastFMConfig {
    token: String,
}

#[derive(Deserialize, Serialize)]
struct ListenbrainzConfig {
    token: String,
}

#[derive(Deserialize, Serialize)]
struct MpdConfig {
    url: Option<String>,
    port: Option<i32>,
    password: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            last_fm: None,
            listenbrainz: None,
            mpd: Some(MpdConfig {
                password: None,
                port: Some(6600),
                url: Some("127.0.0.1".to_owned()),
            }),
        }
    }
}

fn main() {
    env_logger::init();

    let opts: Opts = Opts::parse();

    let config_path = config::get_or_create_config_file(opts.config);
    let config = config::parse(&config_path);

    let mut mpd_client = mpd::MpdClient::new(&get_mpd_url(config.mpd));
    let listenbrainz_client =
        listenbrainz::Client::new(config.listenbrainz.unwrap().token).unwrap();

    loop {
        let response = mpd_client.poll();
        match response.response_type {
            mpd::ResponseType::PlayingNow => {
                let track = listenbrainz::Track {
                    listened_at: None,
                    track_metadata: build_metadata(response.song),
                };

                info!("Submitting now playing: {:#?}", track);
                match listenbrainz_client.submit_now_playing(track) {
                    Ok(_) => info!("Submitted!"),
                    Err(err) => warn!("Could not submit. Error: {:#?}", err),
                }
            }
            mpd::ResponseType::Listen => {
                let track = listenbrainz::Track {
                    listened_at: Some(chrono::Utc::now().timestamp()),
                    track_metadata: build_metadata(response.song),
                };

                info!("Submitting listen: {:#?}", track);
                match listenbrainz_client.submit_listen(track) {
                    Ok(_) => info!("Submitted!"),
                    Err(err) => warn!("Could not submit. Error: {:#?}", err),
                }

                println!("Submitting listen...");
            }
        }
    }
}

fn get_mpd_url(config: Option<MpdConfig>) -> String {
    match config {
        Some(config) => {
            let url = config.url.unwrap_or("127.0.0.1".to_owned());
            let port = config.port.unwrap_or(6600);
            format!("{url}{port}", url = url, port = port)
        }
        None => "127.0.0.1:6600".to_owned(),
    }
}

fn build_metadata(song: mpd::Song) -> listenbrainz::Metadata {
    listenbrainz::Metadata {
        artist_name: song.tags.get("Artist").unwrap_or(&"".to_owned()).to_owned(),
        track_name: song.title.unwrap_or("".to_owned()).to_owned(),
        release_name: song.tags.get("Album").unwrap_or(&"".to_owned()).to_owned(),
        additional_info: None,
    }
}

fn format_elapsed(duration: Duration) -> String {
    let seconds = duration.as_secs();
    let minutes = seconds / 60;
    let seconds_left = seconds - (60 * minutes);
    let ms = duration.subsec_millis();
    format!("{:02}:{:02}.{:3}", minutes, seconds_left, ms)
}
