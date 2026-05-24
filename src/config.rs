use serde::Deserialize;
use std::{
    fs::File,
    io::{self, Read},
};

#[derive(Deserialize)]
pub struct Config {
    pub ws_port: Option<u16>,
    pub ws_password: String,
    pub serial_path: Option<String>,
}

pub fn read_config() -> io::Result<Config> {
    let mut file = File::open("vatromet.toml")?;
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents)?;

    let config: Config = match toml::from_str(&file_contents) {
        Ok(c) => c,
        Err(e) => {
            return Err(io::Error::other(format!("TOML error - {e}")));
        }
    };

    Ok(config)
}
