use crate::rom::Rom;

pub struct Bus {
    pub ram: [u8; 0x800],
    pub rom: Option<Rom>,
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            ram: [0; 0x800],
            rom: None,
        }
    }

    pub fn insert_cartridge(&mut self, rom: Rom) {
        self.rom = Some(rom);
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[(addr & 0x07FF) as usize],
            0x8000.. => match &self.rom {
                Some(rom) => {
                    let rom_index = (addr - 0x8000) as usize;
                    if rom_index < rom.prg_rom.len() {
                        rom.prg_rom[rom_index]
                    } else {
                        panic!("Indexed ROM out of bounds")
                    }
                }
                None => panic!("ROM not loaded yet")
            },
            _ => todo!("unimplemented memory access {}", addr),
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram[(addr & 0x07FF) as usize] = value,
            _ => todo!("unimplemented memory access"),
        }
    }
}
