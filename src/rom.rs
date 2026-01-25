#[derive(Clone)]
pub enum Mirroring {
    Vertical,
    Horizontal,
    FourScreen,
}

pub struct Rom {
    pub header: [u8; 16],
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper: u8,
    pub screen_mirroring: Mirroring,
}

impl Rom {
    pub fn new(raw_bytes: &Vec<u8>) -> Self {
        if raw_bytes.len() < 16 {
            panic!("File is too small");
        }

        let mut header = [0u8; 16];
        header.copy_from_slice(&raw_bytes[0..16]);

        if &header[0..4] != b"NES\x1a" {
            panic!("Not valid iNES file");
        }

        let prg_rom_size = header[4] as usize * 16384;
        let chr_rom_size = header[5] as usize * 8192;

        let flags_6 = raw_bytes[6];
        let flags_7 = raw_bytes[7];

        let prg_rom_start = 16;
        let prg_rom_end = prg_rom_start + prg_rom_size;
        let chr_rom_start = prg_rom_end;
        let chr_rom_end = chr_rom_start + chr_rom_size;

        let prg_rom = raw_bytes[prg_rom_start..prg_rom_end].to_vec();
        let chr_rom = raw_bytes[chr_rom_start..chr_rom_end].to_vec();

        let mapper = (flags_7 & 0xF0) | (flags_6 >> 4);

        let screen_mirroring = if (flags_6 & 1) != 0 {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        };

        Rom {
            header,
            prg_rom,
            chr_rom,
            mapper,
            screen_mirroring,
        }
    }
}
