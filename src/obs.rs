use obws::Client;

use crate::{config::Config, protocol::OU_Packet};

#[derive(Default, Debug)]
pub struct TBarState {
    pub active: bool,
    pub reverse: bool,
}

pub async fn connect(config: &Config) -> Result<Client, obws::error::Error> {
    let port = config.ws_port.unwrap_or(4455);
    Client::connect("127.0.0.1", port, Some(config.ws_password.clone())).await
}

pub async fn handle_tbar(
    state: &mut TBarState,
    packet: &OU_Packet,
    client: &Client,
) -> Result<(), obws::error::Error> {
    let tbar_h = packet.tbar_h;
    let tbar_l = packet.tbar_l;
    let mut position = tbar_h;
    let mut position_precise = u16::from_be_bytes([tbar_h, tbar_l]);
    if state.reverse {
        position = u8::MAX - position;
        position_precise = u16::MAX - position_precise;
    }
    let position_float = position_precise as f32 / u16::MAX as f32;
    if position > 0 || state.active == true {
        let release = if position == 0 || position == 255 {
            true
        } else {
            false
        };
        if release {
            state.active = false;
        } else {
            state.active = true;
        }
        let mut perform_cut = false;
        if position == 255 {
            perform_cut = true;
            state.reverse = !state.reverse;
        }
        client
            .transitions()
            .set_tbar_position(position_float, Some(release))
            .await?;
        if perform_cut {
            client.transitions().trigger().await?;
        }
    }
    Ok(())
}
