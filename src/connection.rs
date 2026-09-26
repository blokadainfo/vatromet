use std::{eprintln, io::{self, Write, stdin, stdout}, num::Wrapping, println};

use midir::{MidiOutput, MidiOutputConnection};

use crate::{config::Config, protocol::OU_Packet};
use anyhow::anyhow;

pub struct Connection {
    midi_connection: Option<MidiOutputConnection>,
    states: States
}

#[derive(Default)]
pub struct States {
    t_bar: u8,
    knobs: [u8; 5],
    knob_directions: [KnobDirection; 5],
}

#[derive(PartialEq, Eq, Debug)]
pub enum KnobDirection {
    Left,
    Right
}

impl Default for KnobDirection {
    fn default() -> Self {
        KnobDirection::Left
    }
}

impl Connection {
    pub fn new(config: &Config) -> anyhow::Result<Connection> {
        Ok(Connection {
            midi_connection: None,
            states: States::default()
        })
    }
    
    pub async fn open(&mut self, config: &Config) -> Result<(), ()> {
        if self.midi_connection.is_none() {
            let midi_out = MidiOutput::new("VATROMETOutput").expect("MidiOutput");

            // Get an output port (read from console if multiple are available)
            let out_ports = midi_out.ports();
            let out_port= match out_ports.len() {
                0 => return Err(()),
                1 => {
                    println!(
                        "Choosing the only available output port: {}",
                        midi_out.port_name(&out_ports[0]).unwrap()
                    );
                    &out_ports[0]
                }
                _ => {
                    println!("\nAvailable output ports:");
                    for (i, p) in out_ports.iter().enumerate() {
                        println!("{}: {}", i, midi_out.port_name(p).unwrap());
                    }
                    print!("Please select output port: ");
                    stdout().flush().unwrap();
                    let mut input = String::new();
                    stdin().read_line(&mut input).unwrap();
                    out_ports
                        .get(input.trim().parse::<usize>().unwrap())
                        .ok_or("invalid output port selected").unwrap()
                }
            };

            println!("\nOpening connection");
            let conn_out = midi_out.connect(out_port, "midir-test").unwrap();
            println!("Connection open. Listen!");
            self.midi_connection = Some(conn_out);
        }
        return Ok(());
    }

    pub async fn handle(
        &mut self,
        packet: &OU_Packet
    ) -> anyhow::Result<()>  {
        let Some(midi_connection) = &mut self.midi_connection else {
            return Err(anyhow!("No midi output connection"));
        };
        // T-Bar
        let tbar_h = packet.tbar_h;
        let tbar_l = packet.tbar_l;
        let position = tbar_h;
        let _position_precise = u16::from_be_bytes([tbar_h, tbar_l]);
        if position != self.states.t_bar {
            self.states.t_bar = position;
            midi_connection.send(&[0xB0, 0x00, position/2])?;
        }

        // Knobs
        for (i, knob) in packet.knobs.iter().enumerate() {
            if self.states.knobs[i] != *knob {
                let difference = (self.states.knobs[i] as i16) - (*knob as i16);
                let direction = {
                    if difference > 128 {
                        KnobDirection::Right
                    } else if difference < -128 {
                        KnobDirection::Left
                    } else if difference > 0 {
                        KnobDirection::Left
                    } else {
                        KnobDirection::Right
                    }
                };
                self.states.knobs[i] = *knob;

                let knob_direction = &mut self.states.knob_directions[i];
                if *knob_direction != direction {
                    *knob_direction = direction;
                    continue;
                }
                
                println!("knob {i} - {knob}");
                let value = {
                    if direction == KnobDirection::Left {
                        64 - 2
                    } else {
                        64 + 2
                    }
                };
               
                midi_connection.send(&[0xB0, 0x10 + i as u8, value])?;
                println!("Direction {:?}", direction);
            }
        }

        // Joystick
        const JOYSTICK_DEADZONE: u16 = 8192;
        const DIVIDER: u16 = 4;
        const HALF: u16 = 32768 - JOYSTICK_DEADZONE;
        for (n, val) in [(0, packet.joystick_x), (1, packet.joystick_y)] {    
            let difference: i8 = {
                if val > JOYSTICK_DEADZONE && val < 32768 {
                    i8::try_from((val - JOYSTICK_DEADZONE) / (HALF / DIVIDER)).unwrap_or(0)
                }
                else if val > 32768 && val < 65535 - JOYSTICK_DEADZONE {
                    let helper = (val as i32) - 65535 + JOYSTICK_DEADZONE as i32;
                    i8::try_from(helper / (HALF / DIVIDER) as i32).unwrap_or(0)
                } else {
                    0
                }
            };
            if difference != 0 {
                midi_connection.send(&[0xB0, 0x20 + n, (64 + difference) as u8])?;
            }
        }

        Ok(())
    }
}