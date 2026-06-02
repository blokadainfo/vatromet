use std::time::Duration;

use obws::Client;
use serialport::SerialPort;
use tokio::time::sleep;

use crate::{
    config::read_config,
    obs::{TBarState, connect, handle_tbar},
    protocol::OU_Packet,
    serial::{open_port, read_frame},
};

mod config;
mod obs;
mod protocol;
mod serial;

#[tokio::main]
async fn main() {
    let config = read_config().expect("Reading config");

    let mut obs_client_option: Option<Client> = None;
    let mut serial_port_option: Option<Box<dyn SerialPort>> = None;

    let mut tbar_state = TBarState::default();

    loop {
        if obs_client_option.is_none() {
            println!("Connecting to OBS...");
            obs_client_option = match connect(&config).await {
                Ok(c) => Some(c),
                Err(e) => {
                    eprintln!("Error connecting to OBS - {e}");
                    sleep(Duration::from_secs(1)).await;
                    None
                }
            };
        }

        let Some(obs_client) = &mut obs_client_option else {
            continue;
        };

        if serial_port_option.is_none() {
            println!("Opening serial port...");
            serial_port_option = match open_port(&config) {
                Ok(p) => Some(p),
                Err(e) => {
                    eprintln!("Error opening port - {e}");
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
                eprintln!("Couldn't read frame - {e}");
                eprintln!("Resetting serial port...");
                serial_port_option = None;
                continue;
            }
        };

        let packet = OU_Packet::from_bytes(&frame);

        if let Err(e) = handle_tbar(&mut tbar_state, &packet, &obs_client).await {
            eprintln!("Setting T-bar failed - {e}");
            eprintln!("Resetting OBS connection...");
            obs_client_option = None;
            continue;
        }
    }
}
