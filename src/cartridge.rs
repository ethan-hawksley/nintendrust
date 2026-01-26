#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    FourScreen,
}

#[derive(Debug, Clone)]
pub struct CartridgeInfo {
    pub mirroring: Mirroring,
    pub has_battery_backed_ram: bool,
    pub has_trainer: bool,
    pub chr_ram_size: usize,
}

impl CartridgeInfo {
    pub fn from_header(header: &[u8; 16]) -> Self {
        let flags_6 = header[6];
        let chr_rom_size = header[5] as usize * 8192;

        let mirroring = if flags_6 & 0x08 != 0 {
            Mirroring::FourScreen
        } else if flags_6 & 0x01 != 0 {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        };

        CartridgeInfo {
            mirroring,
            has_battery_backed_ram: flags_6 & 0x02 != 0,
            has_trainer: flags_6 & 0x04 != 0,
            chr_ram_size: if chr_rom_size == 0 { 8192 } else { 0 },
        }
    }
}
