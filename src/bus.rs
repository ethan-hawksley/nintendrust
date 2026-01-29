use crate::ppu::Ppu;
use crate::rom::Rom;

pub struct Bus {
    pub ram: [u8; 0x800],
    pub rom: Rom,
    pub ppu: Ppu,
}

impl Bus {
    pub fn new(rom: Rom) -> Self {
        let ppu = Ppu::new(rom.cartridge_info.clone(), rom.chr_rom.clone());

        Bus {
            ram: [0; 0x800],
            rom,
            ppu,
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[(addr & 0x07FF) as usize],
            0x2000..=0x3FFF => self.ppu.read_register(addr & 0x2007),
            0x4000..=0x4017 => 0, // TODO: implement audio registers
            0x8000.. => {
                let rom_index = (addr - 0x8000) as usize;
                if rom_index < self.rom.prg_rom.len() {
                    self.rom.prg_rom[rom_index]
                } else {
                    panic!("Indexed ROM out of bounds {}", rom_index);
                }
            }
            _ => todo!("Unimplemented memory access 0x{:04X}", addr),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => self.ram[(address & 0x07FF) as usize] = value,
            0x2000..=0x3FFF => {
                let ppu_address = address & 0x2007;
                self.ppu.write_register(ppu_address, value);
            }
            0x4000..=0x4017 => {} // TODO: implement audio registers
            _ => todo!("Unimplemented memory access 0x{:04X}", address),
        }
    }
}
