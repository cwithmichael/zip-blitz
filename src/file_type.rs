pub trait FileType {
    fn new() -> Self;
    fn get_header(&self) -> Vec<u8>;
    fn is_valid_header(&self, data: &[u8]) -> bool {
        // The size of file signature headers can vary
        let header = self.get_header();
        let bound = std::cmp::min(data.len(), header.len());
        for i in 0..bound {
            if data[i] != header[i] {
                return false;
            }
        }
        true
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Wmv {
    pub header: Vec<u8>,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Jpg {
    pub header: Vec<u8>,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Archive {
    pub header: Vec<u8>,
}

impl FileType for Wmv {
    fn new() -> Wmv {
        Wmv {
            header: vec![
                0x30, 0x26, 0xB2, 0x75, 0x8E, 0x66, 0xCF, 0x11, 0xA6, 0xD9, 0x00, 0xAA, 0x00, 0x62,
                0xCE, 0x6C,
            ],
        }
    }

    fn get_header(&self) -> Vec<u8> {
        return self.header.to_vec();
    }
}

impl FileType for Jpg {
    fn new() -> Jpg {
        Jpg {
            header: vec![0xFF, 0xD8],
        }
    }

    fn get_header(&self) -> Vec<u8> {
        return self.header.to_vec();
    }
}

impl FileType for Archive {
    fn new() -> Archive {
        Archive {
            header: vec![0x50, 0x4B, 0x03, 0x04],
        }
    }

    fn get_header(&self) -> Vec<u8> {
        return self.header.to_vec();
    }
}
