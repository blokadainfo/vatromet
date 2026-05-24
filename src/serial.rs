use std::{io, num::Wrapping, time::Duration};

use serialport::SerialPort;

use crate::config::Config;

const BAUD_RATE: u32 = 307200;
const PORT_TIMEOUT_MS: u64 = 5000;

pub fn open_port(config: &Config) -> serialport::Result<Box<dyn SerialPort>> {
    let path = config
        .serial_path
        .clone()
        .unwrap_or(String::from("/dev/ttyUSB0"));
    serialport::new(path, BAUD_RATE)
        .timeout(Duration::from_millis(PORT_TIMEOUT_MS))
        .open()
}

pub async fn read_frame(serial_port: &mut Box<dyn SerialPort>) -> io::Result<Vec<u8>> {
    let mut frame_started = false;
    let mut frame_contents = Vec::<u8>::new();
    let mut byte_buffer = [0u8; 1];
    loop {
        serial_port.read_exact(&mut byte_buffer)?;
        let byte = byte_buffer[0];
        if frame_started == false && byte != 0x7E {
            continue;
        }
        frame_contents.push(byte);

        if frame_started && byte == 0x7E {
            if frame_contents.len() <= 3 {
                // We've begun reading while a frame was already being transmitted, start over.
                frame_contents = vec![0x7E];
            } else {
                break;
            }
        }
        frame_started = true;
    }
    decode_hdlc_frame(&frame_contents)
}

const FRAME_DELIMITER: u8 = 0x7E;
const FRAME_ESCAPE_CHAR: u8 = 0x7D;

/// Returns decoded payload of frame
pub fn decode_hdlc_frame(packet: &[u8]) -> io::Result<Vec<u8>> {
    if packet[0] != FRAME_DELIMITER || packet[packet.len() - 1] != FRAME_DELIMITER {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Doesn't start/end with delimiter (0x{FRAME_DELIMITER:X})"),
        ));
    }
    let mut decoded_payload = Vec::<u8>::new();
    let mut bit_stuffing = false;
    for byte in &packet[1..packet.len() - 1] {
        if *byte == FRAME_ESCAPE_CHAR {
            bit_stuffing = true;
        } else {
            let decoded_byte = if bit_stuffing {
                bit_stuffing = false;
                *byte ^ 0b1000
            } else {
                *byte
            };
            decoded_payload.push(decoded_byte);
        }
    }
    Ok(decoded_payload)
}

/// Takes HDLC payload and calculates checksum
pub fn calculate_checksum(bytes: &[u8]) -> u8 {
    let mut sum = Wrapping(0u8);
    for byte in &bytes[3..bytes.len() - 1] {
        sum += byte;
    }
    sum -= 50;
    sum.0
}
