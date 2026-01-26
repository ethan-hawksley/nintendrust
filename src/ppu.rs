use crate::cartridge::CartridgeInfo;
use crate::cartridge::Mirroring::FourScreen;
use crate::cartridge::Mirroring::Horizontal;
use crate::cartridge::Mirroring::Vertical;

pub struct Ppu {
    cartridge_info: CartridgeInfo,
    chr_memory: Vec<u8>,
    chr_is_ram: bool,
    vram: [u8; 2048],
    palette_ram: [u8; 32],
    oam: [u8; 256],
    internal_data_buf: u8,
    write_latch: bool,
    vram_address: u16,
    temporary_vram_address: u16,
    transfer_address: u16,
    vram_increment_32: bool,
}

impl Ppu {
    pub fn new(cartridge_info: CartridgeInfo, chr_rom: Vec<u8>) -> Self {
        let (chr_memory, chr_is_ram) = if chr_rom.is_empty() {
            (vec![0; 8192], true)
        } else {
            (chr_rom, false)
        };

        Ppu {
            cartridge_info,
            chr_memory,
            chr_is_ram,
            vram: [0; 2048],
            palette_ram: [0; 32],
            oam: [0; 256],
            internal_data_buf: 0,
            write_latch: false,
            vram_address: 0,
            temporary_vram_address: 0,
            transfer_address: 0,
            vram_increment_32: false,
        }
    }

    pub fn debug_draw_pattern_tables(&self) -> Vec<u8> {
        let width = 256;
        let height = 128;
        let mut frame_buffer = vec![0; width * height * 3];

        let palette = [(0, 0, 0), (85, 85, 85), (170, 170, 170), (255, 255, 255)];

        for table in 0..2 {
            for tile_y in 0..16 {
                for tile_x in 0..16 {
                    let tile_n = tile_y * 16 + tile_x;
                    let offset = table * 4096 + tile_n * 16;

                    if offset + 16 > self.chr_memory.len() {
                        continue;
                    }

                    for row in 0..8 {
                        let tile_lsb = self.chr_memory[offset + row];
                        let tile_msb = self.chr_memory[offset + row + 8];

                        for col in 0..8 {
                            let mask = 1 << (7 - col);
                            let lsb = (tile_lsb & mask) != 0;
                            let msb = (tile_msb & mask) != 0;

                            let val = (if msb { 2 } else { 0 }) | (if lsb { 1 } else { 0 });
                            let (r, g, b) = palette[val];

                            let pixel_x = table * 128 + tile_x * 8 + col;
                            let pixel_y = tile_y * 8 + row;

                            let index = (pixel_y * width + pixel_x) * 3;
                            frame_buffer[index] = r;
                            frame_buffer[index + 1] = g;
                            frame_buffer[index + 2] = b;
                        }
                    }
                }
            }
        }
        frame_buffer
    }

    fn map_vram_address(&self, addr: u16) -> usize {
        let mirrored_addr = addr & 0x0FFF;

        match self.cartridge_info.mirroring {
            Horizontal => ((mirrored_addr & 0x3FF) | ((mirrored_addr >> 1) & 0x400)) as usize,
            Vertical => (mirrored_addr & 0x7FF) as usize,
            FourScreen => {
                todo!("FourScreen mirroring");
            }
        }
    }

    pub fn ppu_data(&mut self, value: u8) {
        match self.vram_address {
            ..0x2000 => {
                // If the CHR ROM is 0-length, it can be used as CHR RAM
                if self.chr_is_ram {
                    self.chr_memory[self.vram_address as usize] = value;
                }
            }
            0x2000..0x3F00 => {
                let mapped_vram_index = self.map_vram_address(self.vram_address);
                self.vram[mapped_vram_index] = value;
            }
            _ => {
                if (self.vram_address & 0x03) == 0 {
                    self.palette_ram[(self.vram_address & 0x0F) as usize] = value;
                } else {
                    self.palette_ram[(self.vram_address & 0x1F) as usize] = value;
                }
            }
        }
        self.vram_address =
            self.vram_address
                .wrapping_add(if self.vram_increment_32 { 32 } else { 1 });
        self.vram_address &= 0x3FFF;
    }

    pub fn ppu_addr(&mut self, value: u8) {
        if !self.write_latch {
            self.temporary_vram_address = ((value & 0x3F) as u16) << 8;
        } else {
            self.vram_address = self.temporary_vram_address | value as u16;
            self.transfer_address = self.vram_address;
        }
        self.write_latch = !self.write_latch;
    }

    pub fn read_register(&mut self, addr: u16) -> u8 {
        match addr {
            _ => 0,
        }
    }

    pub fn write_register(&mut self, addr: u16, _data: u8) {
        match addr {
            _ => {}
        }
    }
}
