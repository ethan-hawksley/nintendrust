pub struct Ppu {
    header: [u8; 16],
    pub chr_rom: Vec<u8>,
    pub internal_data_buf: u8,
    write_latch: bool,
    temporary_vram_address: u16,
    transfer_address: u16,
    vram_address: u16,
}

impl Ppu {
    pub fn new(header: [u8; 16], chr_rom: Vec<u8>) -> Self {
        Ppu {
            header,
            chr_rom,
            internal_data_buf: 0,
            write_latch: false,
            temporary_vram_address: 0,
            transfer_address: 0,
            vram_address: 0,
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

                    if offset + 16 > self.chr_rom.len() {
                        continue;
                    }

                    for row in 0..8 {
                        let tile_lsb = self.chr_rom[offset + row];
                        let tile_msb = self.chr_rom[offset + row + 8];

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

    pub fn ppu_data(&mut self, value: u8) {
        match self.vram_address {
            ..0x2000 => {
                // If the CHR ROM is 0-length, it can be used as CHR RAM
                if self.header[5] == 0 {
                    self.chr_rom[self.vram_address as usize] = value;
                }
            }
            0x2000..0x3F00 => {}
            _ => {}
        }
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
