use std::{env, eprintln, panic, time::Duration};

use serialport::SerialPort;
use tokio::time::sleep;

use crate::{
    config::read_config, connection::{Connection}, protocol::OU_Packet, serial::{open_port, read_frame},
};

mod config;
mod connection;
mod protocol;
mod serial;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config_path = match env::var("VATROMET_CONFIG") {
        Ok(path) => path,
        Err(_) => String::from("vatromet.toml"),
    };

    let config = match read_config(config_path) {
        Ok(c) => c,
        Err(e) => {
            panic!("Couldn't load configuration file. {e}");
        }
    };

    let mut connection = Connection::new(&config)?;
    let mut serial_port_option: Option<Box<dyn SerialPort>> = None;

    loop {
        if let Err(e) = connection.open().await {
            eprintln!("Couldn't open connection. {e}");
            sleep(Duration::from_secs(1)).await;
            continue;
        }

        if serial_port_option.is_none() {
            println!("Opening serial port...");
            serial_port_option = match open_port(&config) {
                Ok(p) => Some(p),
                Err(e) => {
                    eprintln!("Couldn't open port. {e}");
                    sleep(Duration::from_secs(1)).await;
                    None
                }
            };
        }

        let Some(serial_port) = &mut serial_port_option else {
            continue;
        };

        let frame = match read_frame(serial_port).await {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Couldn't read frame. {e}");
                eprintln!("Resetting serial port...");
                serial_port_option = None;
                continue;
            }
        };

        let packet = OU_Packet::from_bytes(&frame);

        if let Err(e) = connection.handle(&packet).await {
            eprintln!("Couldn't handle packet. {e}");
        }


    }
}
