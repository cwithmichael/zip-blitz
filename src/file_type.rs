pub trait FileType<'a> {
    fn new() -> Self;
    fn get_header(&self) -> &'a [u8];
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

pub struct Wmv<'a> {
    header: &'a [u8],
}

pub struct Jpg<'a> {
    header: &'a [u8],
}

pub struct Archive<'a> {
    header: &'a [u8],
}

impl<'a> FileType<'a> for Wmv<'a> {
    fn new() -> Wmv<'a> {
        Wmv {
            header: &[
                0x30, 0x26, 0xB2, 0x75, 0x8E, 0x66, 0xCF, 0x11, 0xA6, 0xD9, 0x00, 0xAA, 0x00, 0x62,
                0xCE, 0x6C,
            ],
        }
    }

    fn get_header(&self) -> &'a [u8] {
        return self.header;
    }
}

impl<'a> FileType<'a> for Jpg<'a> {
    fn new() -> Jpg<'a> {
        Jpg {
            header: &[0xFF, 0xD8],
        }
    }

    fn get_header(&self) -> &'a [u8] {
        return self.header;
    }
}

impl<'a> FileType<'a> for Archive<'a> {
    fn new() -> Archive<'a> {
        Archive {
            header: &[0x50, 0x4B, 0x03, 0x04],
        }
    }

    fn get_header(&self) -> &'a [u8] {
        return self.header;
    }
}
