use crate::ppu::Ppu;
use crate::rom::Rom;

pub struct Bus {
    pub ram: [u8; 0x800],
    pub rom: Rom,
    pub ppu: Ppu,
}

impl Bus {
    pub fn new(rom: Rom) -> Self {
        let ppu = Ppu::new(rom.chr_rom.clone(), rom.screen_mirroring.clone());

        Bus {
            ram: [0; 0x800],
            rom,
            ppu,
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[(addr & 0x07FF) as usize],
            0x8000.. => {
                let rom_index = (addr - 0x8000) as usize;
                if rom_index < self.rom.prg_rom.len() {
                    self.rom.prg_rom[rom_index]
                } else {
                    panic!("Indexed ROM out of bounds {}", rom_index);
                }
            }
            _ => todo!("Unimplemented memory access {}", addr),
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram[(addr & 0x07FF) as usize] = value,
            _ => todo!("Unimplemented memory access {}", addr),
        }
    }
}
