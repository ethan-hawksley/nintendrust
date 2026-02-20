use crate::controller::Controller;
use crate::ppu::Ppu;
use crate::rom::Rom;

pub struct Bus {
    pub ram: [u8; 0x800],
    pub rom: Rom,
    pub ppu: Ppu,
    pub controller: Controller,
}

impl Bus {
    pub fn new(rom: Rom) -> Self {
        let ppu = Ppu::new(rom.cartridge_info.clone(), rom.chr_rom.clone());
        let controller = Controller::new();

        Bus {
            ram: [0; 0x800],
            rom,
            ppu,
            controller,
        }
    }

    pub fn peek(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[(addr & 0x07FF) as usize],
            0x2000..=0x3FFF => self.ppu.peek_register(addr & 0x2007),
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

    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[(addr & 0x07FF) as usize],
            0x2000..=0x3FFF => self.ppu.read_register(addr & 0x2007),
            0x4016 => {
                let controller_bit = (self.controller.shift_register_1 & 0x80) >> 7;
                self.controller.shift_register_1 <<= 1;
                controller_bit
            }
            0x4017 => {
                let controller_bit = (self.controller.shift_register_2 & 0x80) >> 7;
                self.controller.shift_register_2 <<= 1;
                controller_bit
            }
            0x4000..=0x4015 => 0,
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
            0x4014 => {
                // OAM DMA
                for i in 0..256 {
                    self.ppu.oam[i] = self.read(((value as u16) << 8) + i as u16)
                }
            }
            0x4016 => {
                self.controller.shift_register_1 = self.controller.controller1;
                self.controller.shift_register_2 = self.controller.controller2;
            }
            0x4000..=0x4013 | 0x4015..=0x4017 => {
                // TODO: implement audio and controller latching
            }
            _ => todo!("Unimplemented memory access 0x{:04X}", address),
        }
    }
}
