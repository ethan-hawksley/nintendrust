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
    write_latch: bool,
    vram_address: u16,
    temporary_vram_address: u16,
    transfer_address: u16,
    vram_increment_32: bool,
    read_buffer: u8,
    ppu_dot: u16,
    ppu_scanline: u16,
    pub v_blank: bool,
    mask_background_8px: bool,
    mask_sprites_8px: bool,
    mask_render_background: bool,
    mask_render_sprites: bool,
    nametable_select: u8,
    sprite_pattern_table: bool,
    bg_pattern_table: bool,
    use_8x16_sprites: bool,
    pub enable_nmi: bool,
    shift_register_pattern_l: u16,
    shift_register_pattern_h: u16,
    shift_register_attribute_l: u16,
    shift_register_attribute_h: u16,
    cycle_attribute: u8,
    pattern_low_bit_plane: u8,
    pattern_high_bit_plane: u8,
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
            write_latch: false,
            vram_address: 0,
            temporary_vram_address: 0,
            transfer_address: 0,
            vram_increment_32: false,
            read_buffer: 0,
            ppu_dot: 0,
            ppu_scanline: 0,
            v_blank: false,
            mask_background_8px: false,
            mask_sprites_8px: false,
            mask_render_background: false,
            mask_render_sprites: false,
            nametable_select: 0,
            sprite_pattern_table: false,
            bg_pattern_table: false,
            use_8x16_sprites: false,
            enable_nmi: false,
            shift_register_pattern_l: 0,
            shift_register_pattern_h: 0,
            shift_register_attribute_l: 0,
            shift_register_attribute_h: 0,
            cycle_attribute: 0,
            pattern_low_bit_plane: 0,
            pattern_high_bit_plane: 0,
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

    pub fn debug_draw_nametable(&self) -> Vec<u8> {
        // Draw 2 nametables side by side
        // Each nametable is 256x240 pixels (32x30 tiles of 8x8 pixels)
        let width = 512;
        let height = 240;
        let mut frame_buffer = vec![0; width * height * 3];

        let palette = [
            // Row 0
            (0x65, 0x65, 0x65),
            (0x00, 0x2A, 0x84),
            (0x15, 0x13, 0xA2),
            (0x3A, 0x01, 0x9E),
            (0x59, 0x00, 0x7A),
            (0x6A, 0x00, 0x3E),
            (0x68, 0x08, 0x00),
            (0x53, 0x1D, 0x00),
            (0x32, 0x34, 0x00),
            (0x0D, 0x46, 0x00),
            (0x00, 0x4F, 0x00),
            (0x00, 0x4C, 0x09),
            (0x00, 0x3F, 0x4B),
            (0x00, 0x00, 0x00),
            (0x00, 0x00, 0x00),
            (0x00, 0x00, 0x00),
            // Row 1
            (0xAE, 0xAE, 0xAE),
            (0x17, 0x5F, 0xD6),
            (0x43, 0x41, 0xFF),
            (0x75, 0x29, 0xFA),
            (0x9E, 0x1D, 0xCA),
            (0xB4, 0x20, 0x7B),
            (0xB1, 0x33, 0x22),
            (0x96, 0x4E, 0x00),
            (0x6A, 0x6C, 0x00),
            (0x39, 0x84, 0x00),
            (0x0F, 0x90, 0x00),
            (0x00, 0x8D, 0x33),
            (0x00, 0x7B, 0x8C),
            (0x00, 0x00, 0x00),
            (0x00, 0x00, 0x00),
            (0x00, 0x00, 0x00),
            // Row 2
            (0xFE, 0xFE, 0xFE),
            (0x66, 0xAF, 0xFF),
            (0x93, 0x90, 0xFF),
            (0xC5, 0x78, 0xFF),
            (0xEE, 0x6C, 0xFF),
            (0xFF, 0x6F, 0xCA),
            (0xFF, 0x82, 0x71),
            (0xE6, 0x9E, 0x25),
            (0xBA, 0xBC, 0x00),
            (0x88, 0xD5, 0x01),
            (0x5E, 0xE1, 0x32),
            (0x47, 0xDD, 0x82),
            (0x4A, 0xCB, 0xDC),
            (0x4E, 0x4E, 0x4E),
            (0x00, 0x00, 0x00),
            (0x00, 0x00, 0x00),
            // Row 3
            (0xFE, 0xFE, 0xFE),
            (0xC0, 0xDE, 0xFF),
            (0xD2, 0xD1, 0xFF),
            (0xE7, 0xC7, 0xFF),
            (0xF8, 0xC2, 0xFF),
            (0xFF, 0xC3, 0xE9),
            (0xFF, 0xCB, 0xC4),
            (0xF5, 0xD7, 0xA5),
            (0xE2, 0xE3, 0x94),
            (0xCE, 0xED, 0x96),
            (0xBC, 0xF2, 0xAA),
            (0xB3, 0xF1, 0xCB),
            (0xB4, 0xE9, 0xF0),
            (0xB6, 0xB6, 0xB6),
            (0x00, 0x00, 0x00),
            (0x00, 0x00, 0x00),
        ];

        let pattern_table_base = if self.bg_pattern_table { 4096 } else { 0 };

        for nametable in 0..2u16 {
            let nametable_base = 0x2000 + nametable * 0x400;
            let attribute_base = nametable_base + 0x3C0;
            let screen_offset_x = nametable as usize * 256;

            for tile_y in 0..30usize {
                for tile_x in 0..32usize {
                    // Get tile index from nametable in VRAM
                    let nametable_addr = nametable_base + (tile_y * 32 + tile_x) as u16;
                    let mapped_addr = self.map_vram_address(nametable_addr);
                    let tile_index = self.vram[mapped_addr as usize] as usize;

                    let attr_offset = (tile_y / 4) * 8 + (tile_x / 4);
                    let attr_addr = attribute_base + attr_offset as u16;
                    let mapped_attr_addr = self.map_vram_address(attr_addr);
                    let attr_byte = self.vram[mapped_attr_addr as usize];

                    let shift = ((tile_y & 2) << 1) | (tile_x & 2);
                    let palette_idx = (attr_byte >> shift) & 0x3;

                    // Get tile from pattern table 0
                    let chr_offset = tile_index * 16;

                    for row in 0..8 {
                        if chr_offset + row + 8 >= self.chr_memory.len() {
                            continue;
                        }

                        let tile_lsb = self.chr_memory[chr_offset + row + pattern_table_base];
                        let tile_msb = self.chr_memory[chr_offset + row + 8 + pattern_table_base];

                        for col in 0..8 {
                            let mask = 1 << (7 - col);
                            let lsb = if tile_lsb & mask != 0 { 1 } else { 0 };
                            let msb = if tile_msb & mask != 0 { 2 } else { 0 };
                            let pixel_val = msb | lsb;

                            let color_index_in_palette_ram = if pixel_val == 0 {
                                0
                            } else {
                                (palette_idx as usize * 4) + pixel_val
                            };

                            let color_index = self.palette_ram[color_index_in_palette_ram] as usize;

                            let (r, g, b) = palette[color_index];

                            let pixel_x = screen_offset_x + tile_x * 8 + col;
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

    fn map_vram_address(&self, addr: u16) -> u16 {
        let mirrored_addr = addr & 0x0FFF;

        match self.cartridge_info.mirroring {
            Horizontal => (mirrored_addr & 0x3FF) | ((mirrored_addr >> 1) & 0x400),
            Vertical => mirrored_addr & 0x7FF,
            FourScreen => {
                todo!("FourScreen mirroring");
            }
        }
    }

    pub fn peek_register(&self, address: u16) -> u8 {
        match address {
            0x2002 => (self.v_blank as u8) << 7, // PPU STATUS
            0x2007 => self.read_buffer,
            _ => 0,
        }
    }

    fn read_ppu(&self, address: u16) -> u8 {
        match address {
            ..0x2000 => {
                // Read from pattern table
                self.chr_memory[self.vram_address as usize]
            }
            0x2000..0x3F00 => {
                // Read from nametables
                let mapped_vram_address = self.map_vram_address(self.vram_address);
                self.vram[mapped_vram_address as usize]
            }
            0x3F00.. => {
                if (self.vram_address & 3) == 0 {
                    self.palette_ram[(self.vram_address & 0x0F) as usize]
                } else {
                    self.palette_ram[(self.vram_address & 0x1F) as usize]
                }
            }
        }
    }

    pub fn read_register(&mut self, address: u16) -> u8 {
        match address {
            0x2002 => {
                // PPU STATUS
                let status = (self.v_blank as u8) << 7 | 1 << 6;
                self.v_blank = false;
                self.write_latch = false;
                status
            }
            0x2007 => {
                // PPU DATA
                let mut previous_buffer = self.read_buffer;

                if self.vram_address >= 0x3f00 {
                    previous_buffer = self.read_ppu(address);
                } else {
                    self.read_buffer = self.read_ppu(address);
                }

                self.vram_address =
                    self.vram_address
                        .wrapping_add(if self.vram_increment_32 { 32 } else { 1 });
                self.vram_address &= 0x3FFF;
                previous_buffer
            }
            _ => 0,
        }
    }

    pub fn write_register(&mut self, address: u16, value: u8) {
        match address {
            0x2000 => {
                // PPU CTRL
                self.nametable_select = value & 3;
                self.vram_increment_32 = value & 4 != 0;
                self.sprite_pattern_table = value & 8 != 0;
                self.bg_pattern_table = value & 0x10 != 0;
                self.use_8x16_sprites = value & 0x20 != 0;
                self.enable_nmi = value & 0x80 != 0;
            }
            0x2001 => {
                // PPU MASK
                self.mask_background_8px = (value & 2) != 0;
                self.mask_sprites_8px = (value & 4) != 0;
                self.mask_render_background = (value & 8) != 0;
                self.mask_render_sprites = (value & 0x10) != 0;
            }
            0x2002 => {}
            0x2003 => {}
            0x2004 => {}
            0x2005 => {}
            0x2006 => {
                // PPU ADDR
                self.ppu_addr(value);
            }
            0x2007 => {
                // PPU DATA
                self.ppu_data(value);
            }
            _ => {
                todo!("Unimplemented ppu register write 0x{:04X}", address);
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
                self.vram[mapped_vram_index as usize] = value;
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

    pub fn emulate_ppu(&mut self) {
        if self.ppu_dot == 1 && self.ppu_scanline == 241 {
            self.v_blank = true;
        } else if self.ppu_dot == 1 && self.ppu_scanline == 261 {
            self.v_blank = false;
        }

        if self.ppu_scanline < 240 || self.ppu_scanline == 261 {
            // If this is a visible scanline, or the pre-render line.
            if (self.ppu_dot > 0 && self.ppu_dot <= 256)
                || (self.ppu_dot > 320 && self.ppu_dot <= 336)
            {
                // If this is a visible pixel, or preparing the start of the next scanline.
                if self.mask_render_background || self.mask_render_sprites {
                    // If rendering is enabled.
                    if self.mask_render_background {
                        // If rendering the background, update the shift registers for the background.
                        // Shift registers one bit to the left.
                        self.shift_register_pattern_l <<= 1;
                        self.shift_register_pattern_h <<= 1;
                        self.shift_register_attribute_l <<= 1;
                        self.shift_register_attribute_h <<= 1;
                    }
                }
            }
        }

        self.ppu_dot += 1;

        if self.ppu_dot >= 341 {
            self.ppu_dot = 0;
            self.ppu_scanline += 1;
            if self.ppu_scanline >= 262 {
                self.ppu_scanline = 0;
            }
        }

        let cycle_tick: u8 = ((self.ppu_dot - 1) & 7) as u8;
        match cycle_tick {
            0 => {
                self.shift_register_pattern_l =
                    (self.shift_register_pattern_l & 0xff00) | self.pattern_low_bit_plane as u16;
                self.shift_register_pattern_h =
                    (self.shift_register_pattern_h & 0xff00) | self.pattern_high_bit_plane as u16;
                self.shift_register_attribute_l = (self.shift_register_attribute_l & 0xff00)
                    | if (self.cycle_attribute & 1) == 1 {
                        0xff
                    } else {
                        0
                    };
                self.shift_register_attribute_h = (self.shift_register_attribute_h & 0xff00)
                    | if (self.cycle_attribute & 2) == 2 {
                        0xff
                    } else {
                        0
                    };
            }
            1 => {}
            2 => {}
            3 => {}
            4 => {}
            5 => {}
            6 => {}
            7 => {}
            _ => unreachable!(),
        };
    }
}
