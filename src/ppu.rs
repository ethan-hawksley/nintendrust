use crate::rom::Mirroring;

pub struct Ppu {
    pub chr_rom: Vec<u8>,
    pub mirroring: Mirroring,
    pub internal_data_buf: u8,
}

impl Ppu {
    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        Ppu {
            chr_rom,
            mirroring,
            internal_data_buf: 0,
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
