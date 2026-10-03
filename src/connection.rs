
use midir::{MidiOutput, MidiOutputConnection};

use crate::{config::Config, protocol::OU_Packet};
use anyhow::anyhow;

pub struct Connection {
    midi_connection: Option<MidiOutputConnection>,
    midi_name: String,
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
            midi_name: config.midi_name.clone(),
            states: States::default()
        })
    }
    
    pub async fn open(&mut self) -> anyhow::Result<()> {
        if self.midi_connection.is_none() {
            let midi_out = MidiOutput::new("VATROMETOutput")?;
            let out_ports = midi_out.ports();
            let mut matched_port= None;
            for port in &out_ports {
                let Ok(port_name) = midi_out.port_name(port) else {
                    continue;
                };
                if port_name.contains(self.midi_name.as_str()) {
                    matched_port = Some(port);
                    break;
                }
            }
            let Some(out_port) = matched_port else {
                return Err(anyhow!("No port with matched name '{}' found", self.midi_name))
            };

            let Ok(conn_out) = midi_out.connect(out_port, "VATROMET_CONN") else {
                return Err(anyhow!("Couldn't connect to MIDI output port"));
            };
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
                
                let value = {
                    if direction == KnobDirection::Left {
                        64 - 2
                    } else {
                        64 + 2
                    }
                };
               
                midi_connection.send(&[0xB0, 0x10 + i as u8, value])?;
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