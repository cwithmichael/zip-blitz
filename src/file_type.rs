#[derive(PartialEq, Eq, Debug)]
pub struct Wmv {
    pub header: Vec<u8>,
}

impl Default for Wmv {
    fn default() -> Wmv {
        Wmv {
            header: vec![
                0x30, 0x26, 0xB2, 0x75, 0x8E, 0x66, 0xCF, 0x11, 0xA6, 0xD9, 0x00, 0xAA, 0x00, 0x62,
                0xCE, 0x6C,
            ],
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Jpg {
    pub header: Vec<u8>,
}

impl Default for Jpg {
    fn default() -> Jpg {
        Jpg {
            header: vec![0xFF, 0xD8],
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Archive {
    pub header: Vec<u8>,
}

impl Default for Archive {
    fn default() -> Archive {
        Archive {
            header: vec![0x50, 0x4B, 0x03, 0x04],
        }
    }
}
