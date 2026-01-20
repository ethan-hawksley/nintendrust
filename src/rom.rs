pub struct Rom {
    pub header: [u8; 16],
    pub prg_rom: Vec<u8>,
}

impl Rom {
    pub fn new(raw_bytes: &Vec<u8>) -> Result<Self, String> {
        if raw_bytes.len() < 16 {
            return Err("File is too small".to_string());
        }

        let mut header = [0u8; 16];
        header.copy_from_slice(&raw_bytes[0..16]);

        if &header[0..4] != b"NES\x1a" {
            return Err("Not valid iNES file".to_string())
        }

        let prg_rom = raw_bytes[16..].to_vec();

        Ok(Rom {
            header,
            prg_rom
        })
    }
}
