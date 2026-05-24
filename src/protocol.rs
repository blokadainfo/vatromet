#[derive(Debug, Default)]
pub struct OU_Packet {
    pub counter: u8,
    pub joystick_x: u16,
    pub joystick_y: u16,
    pub tbar_h: u8,
    pub tbar_l: u8,
    pub knob1: u8,
    pub knob2: u8,
    pub knob3: u8,
    pub knob4: u8,
    pub size_knob: u8,
    pub buzzer: u8,
    pub screensaver: u8,
    pub brightness: u8,
    pub raw_bytes: Vec<u8>,
    pub checksum: u8,
    pub flags1: u8,
    pub flags2: u8,
}

impl OU_Packet {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let raw_bytes = bytes.to_vec();
        if bytes.len() <= 41 {
            return Self {
                raw_bytes,
                ..Default::default()
            };
        }
        let counter = bytes[2];
        let joystick_x = u16::from_be_bytes([bytes[24], bytes[25]]);
        let joystick_y = u16::from_be_bytes([bytes[26], bytes[27]]);
        let tbar_h = bytes[28];
        let tbar_l = bytes[29];
        let knob1 = bytes[30];
        let knob2 = bytes[31];
        let knob3 = bytes[32];
        let knob4 = bytes[33];
        let size_knob = bytes[34];
        let buzzer = bytes[37];
        let brightness = bytes[38];
        let screensaver = bytes[39];
        let checksum = bytes[bytes.len() - 1];
        let flags1 = bytes[23];
        let flags2 = bytes[41];

        Self {
            counter,
            joystick_x,
            joystick_y,
            tbar_h,
            tbar_l,
            knob1,
            knob2,
            knob3,
            knob4,
            size_knob,
            buzzer,
            brightness,
            screensaver,
            raw_bytes,
            checksum,
            flags1,
            flags2,
        }
    }
}
