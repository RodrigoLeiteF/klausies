use crate::Config;
use std::fs;
use std::path::PathBuf;
use toml;
use xdg::BaseDirectories;

pub fn parse(path: &PathBuf) -> Config {
    let data = fs::read_to_string(path).expect("Could not read config file");
    let config: Config = toml::from_str(&data).expect("Could not parse config file");

    match config.listenbrainz {
        None => panic!("No listenbrainz token found"),
        _ => (),
    }

    config
}

pub fn get_or_create_config_file(path: Option<String>) -> PathBuf {
    let xdg_dirs = BaseDirectories::with_prefix("scrabbler").unwrap();

    if let Some(dir) = path {
        let path = PathBuf::from(dir);
        fs::canonicalize(path).expect("The provided config file's path could not be extended")
    } else {
        let path = xdg_dirs
            .place_config_file("config.toml")
            .expect("Cannot create config file");
        if !path.exists() {
            let default = Config {
                ..Default::default()
            };
            let default_str = toml::to_string_pretty(&default).unwrap();
            println!("{:?}", default_str);
            std::fs::write(&path, default_str).expect("Cannot write config file");
        }

        path
    }
}
