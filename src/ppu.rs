use crate::cartridge::CartridgeInfo;
use crate::cartridge::Mirroring::FourScreen;
use crate::cartridge::Mirroring::Horizontal;
use crate::cartridge::Mirroring::Vertical;
use crate::palette::NES_PALETTE;

pub struct Ppu {
    cartridge_info: CartridgeInfo,
    chr_memory: Vec<u8>,
    chr_is_ram: bool,
    vram: [u8; 2048],
    palette_ram: [u8; 32],
    pub oam: [u8; 256],
    write_latch: bool,
    vram_address: u16,
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
    address_bus: u16,
    cycle_temp: u8,
    cycle_next_character: u8,
    ppu_scroll_fine_x: u8,
    pub frame_buffer: [u32; 256 * 240],
    pub frame_complete: bool,
    secondary_oam: [u8; 32],
    sprite_evaluation_temp: u8,
    secondary_oam_address: u8,
    secondary_oam_full: bool,
    oam_address: u16,
    sprite_evaluation_tick: u8,
    status_overflow: bool,
    status_sprite_zero_hit: bool,
    scanline_contains_sprite_zero: bool,
    sprite_evaluation_oam_overflowed: bool,
    pub secondary_oam_size: u8,
    pub sprite_shift_register_l: [u8; 8],
    pub sprite_shift_register_h: [u8; 8],
    pub sprite_attribute: [u8; 8],
    pub sprite_pattern: [u8; 8],
    pub sprite_x_position: [u8; 8],
    pub sprite_y_position: [u8; 8],
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
            address_bus: 0,
            cycle_temp: 0,
            cycle_next_character: 0,
            ppu_scroll_fine_x: 0,
            frame_buffer: [0; 256 * 240],
            frame_complete: false,
            secondary_oam: [0; 32],
            sprite_evaluation_temp: 0,
            secondary_oam_address: 0,
            secondary_oam_full: false,
            oam_address: 0,
            sprite_evaluation_tick: 0,
            status_overflow: false,
            status_sprite_zero_hit: false,
            scanline_contains_sprite_zero: false,
            sprite_evaluation_oam_overflowed: false,
            secondary_oam_size: 0,
            sprite_shift_register_l: [0; 8],
            sprite_shift_register_h: [0; 8],
            sprite_attribute: [0; 8],
            sprite_pattern: [0; 8],
            sprite_x_position: [0; 8],
            sprite_y_position: [0; 8],
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

                            let color_index =
                                (self.palette_ram[color_index_in_palette_ram] & 0x3f) as usize;

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
                self.chr_memory[address as usize]
            }
            0x2000..0x3F00 => {
                // Read from nametables
                let mapped_vram_address = self.map_vram_address(address);
                self.vram[mapped_vram_address as usize]
            }
            0x3F00.. => {
                if (address & 3) == 0 {
                    self.palette_ram[(address & 0x0F) as usize]
                } else {
                    self.palette_ram[(address & 0x1F) as usize]
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
                    previous_buffer = self.read_ppu(self.vram_address);
                } else {
                    self.read_buffer = self.read_ppu(self.vram_address);
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

                self.transfer_address =
                    (self.transfer_address & 0xf3ff) | ((value as u16 & 0x03) << 10);
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
            0x2005 => {
                // PPU Scroll
                if !self.write_latch {
                    self.ppu_scroll_fine_x = value & 7;
                    self.transfer_address =
                        (self.transfer_address & 0xffe0) | ((value >> 3) as u16);
                } else {
                    self.transfer_address = (self.transfer_address & 0x8c1f)
                        | ((((value as u16) & 0xf8) << 2) | (((value as u16) & 0x07) << 12))
                }

                self.write_latch = !self.write_latch;
            }
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
            self.transfer_address =
                (self.transfer_address & 0x00ff) | (((value as u16) & 0x3f) << 8);
        } else {
            self.transfer_address = (self.transfer_address & 0xFF00) | (value as u16);
            self.vram_address = self.transfer_address;
        }
        self.write_latch = !self.write_latch;
    }

    fn increment_y_scroll(&mut self) {
        if (self.vram_address & 0x7000) != 0x7000 {
            self.vram_address = self.vram_address.wrapping_add(0x1000);
        } else {
            self.vram_address &= 0x0fff;

            let mut y = (self.vram_address & 0x03E0) >> 5;

            if y == 29 {
                // Reset the Y value and flip bit 11 of VRAM address.
                y = 0;
                self.vram_address ^= 0x0800;
            } else {
                y = y.wrapping_add(1);
                y &= 0x1f;
            }

            self.vram_address = (self.vram_address & 0xfc1f) | (y << 5);
        }
    }

    fn reset_x_scroll(&mut self) {
        self.vram_address =
            (self.vram_address & 0b0111101111100000) | (self.transfer_address & 0b0000010000011111);
    }

    fn reset_y_scroll(&mut self) {
        self.vram_address =
            (self.vram_address & 0b0000010000011111) | (self.transfer_address & 0b0111101111100000);
    }

    fn sprite_evaluation(&mut self) {
        if self.ppu_dot == 0 {
            self.secondary_oam_address = 0;
            self.secondary_oam_full = false;
        } else if self.ppu_dot > 0 && self.ppu_dot <= 64 {
            if (self.ppu_dot & 1) == 1 {
                // Odd PPU cycles load the value $FF
                self.sprite_evaluation_temp = 0xff;
            } else {
                // Even PPU cycles store the value in Secondary OAM
                self.secondary_oam[self.secondary_oam_address as usize] =
                    self.sprite_evaluation_temp;
                self.secondary_oam_address += 1;
                // Address stays between $00 and $1f
                self.secondary_oam_address &= 0x1f;
            }
        } else if self.ppu_dot > 64 && self.ppu_dot <= 256 {
            if (self.ppu_dot & 1) == 1 {
                // Odd PPU cycles load the value from OAM.
                self.sprite_evaluation_temp = self.oam[self.oam_address as usize]
            } else {
                if (!self.sprite_evaluation_oam_overflowed) {
                    if !self.secondary_oam_full {
                        // If secondary OAM isn't full, the write always occurs.
                        self.secondary_oam[self.secondary_oam_address as usize] =
                            self.sprite_evaluation_temp;
                    }
                }
                // Even PPU cycles store the value in secondary OAM.
                if self.sprite_evaluation_tick == 0 {
                    // Reading index 0 of an object's set of four bytes.
                    if self.ppu_scanline >= self.sprite_evaluation_temp as u16
                        && (self.ppu_scanline - self.sprite_evaluation_temp as u16)
                            < (if self.use_8x16_sprites { 16 } else { 8 })
                    {
                        // This object is on the scanline.
                        if !self.secondary_oam_full {
                            // Increment for next write to secondary OAM.
                            self.secondary_oam_address += 1;
                            // Increment for next read from OAM.
                            self.oam_address += 1;
                            if self.ppu_dot == 66 {
                                // Index 0 is evaluated on PPU dot 66
                                self.scanline_contains_sprite_zero = true;
                            }
                        } else {
                            self.status_overflow = true;
                        }
                        self.sprite_evaluation_tick += 1;
                    } else {
                        self.oam_address += 4;
                    }
                } else {
                    // Reading index 1, 2, or 3 of an object's OAM data.
                    // Increment for next write to secondary OAM
                    self.secondary_oam_address += 1;
                    // Increment for next read from OAM
                    self.oam_address += 1;

                    if self.secondary_oam_address == 0x20 {
                        self.secondary_oam_full = true;
                    }
                    self.sprite_evaluation_tick += 1;
                    // Wrap to tick 0 after tick 3.
                    self.sprite_evaluation_tick &= 3;
                }
                if self.oam_address == 0 {
                    // If OAM overflowed, stop running sprite evaluation until dot 257.
                    self.sprite_evaluation_oam_overflowed = true;
                }
            }
        } else if self.ppu_dot > 256 && self.ppu_dot <= 320 {
            // Address reset to $00 during every one of these cycles.
            self.oam_address = 0;
            if self.ppu_dot == 257 {
                self.secondary_oam_size = self.secondary_oam_address;
                self.secondary_oam_address = 0;
                self.sprite_evaluation_tick = 0;
            }
            match self.sprite_evaluation_tick {
                0 => {
                    // Set this object's Y position in the array.
                    self.sprite_y_position[(self.secondary_oam_address / 4) as usize] =
                        self.secondary_oam[self.secondary_oam_address as usize];
                    self.secondary_oam_address += 1;
                }
                1 => {
                    // Set this object's pattern in the array.
                    self.sprite_pattern[(self.secondary_oam_address / 4) as usize] =
                        self.secondary_oam[self.secondary_oam_address as usize];
                    self.secondary_oam_address += 1;
                }
                2 => {
                    // Set this object's attributes in the array.
                    self.sprite_attribute[(self.secondary_oam_address / 4) as usize] =
                        self.secondary_oam[self.secondary_oam_address as usize];
                    self.secondary_oam_address += 1;
                }
                3 => {
                    // Set this object's X position in the array.
                    self.sprite_attribute[(self.secondary_oam_address / 4) as usize] =
                        self.secondary_oam[self.secondary_oam_address as usize];
                }
                4 => {}
                5 => {}
                6 => {}
                7 => {}
                _ => unreachable!(),
            }
            self.sprite_evaluation_tick += 1;
            // Reset tick at 8.
            self.sprite_evaluation_tick &= 7;
        }
    }

    pub fn emulate_ppu(&mut self) {
        let rendering_enabled = self.mask_render_background || self.mask_render_sprites;
        let visible_or_prerender = self.ppu_scanline < 240 || self.ppu_scanline == 261;
        let fetching_dot = (self.ppu_dot > 0 && self.ppu_dot <= 256)
            || (self.ppu_dot > 320 && self.ppu_dot <= 336);

        if visible_or_prerender && rendering_enabled {
            if fetching_dot {
                if self.mask_render_background {
                    self.shift_register_pattern_l <<= 1;
                    self.shift_register_pattern_h <<= 1;
                    self.shift_register_attribute_l <<= 1;
                    self.shift_register_attribute_h <<= 1;
                }

                let cycle_tick: u8 = (self.ppu_dot.wrapping_sub(1) & 7) as u8;
                match cycle_tick {
                    0 => {
                        self.shift_register_pattern_l = (self.shift_register_pattern_l & 0xff00)
                            | self.pattern_low_bit_plane as u16;
                        self.shift_register_pattern_h = (self.shift_register_pattern_h & 0xff00)
                            | self.pattern_high_bit_plane as u16;
                        self.shift_register_attribute_l = (self.shift_register_attribute_l
                            & 0xff00)
                            | if (self.cycle_attribute & 1) == 1 {
                                0xff
                            } else {
                                0
                            };
                        self.shift_register_attribute_h = (self.shift_register_attribute_h
                            & 0xff00)
                            | if (self.cycle_attribute & 2) == 2 {
                                0xff
                            } else {
                                0
                            };

                        self.address_bus = 0x2000 + (self.vram_address & 0x0FFF);
                        self.cycle_temp = self.read_ppu(self.address_bus);
                    }
                    1 => {
                        self.cycle_next_character = self.cycle_temp;
                    }
                    2 => {
                        self.address_bus = 0x23C0
                            | (self.vram_address & 0x0C00)
                            | ((self.vram_address >> 4) & 0x38)
                            | ((self.vram_address >> 2) & 0x07);
                        self.cycle_temp = self.read_ppu(self.address_bus);
                    }
                    3 => {
                        self.cycle_attribute = self.cycle_temp;
                        if (self.vram_address & 3) >= 2 {
                            self.cycle_attribute >>= 2;
                        }
                        if (((self.vram_address & 0b0000001111100000) >> 5) & 3) >= 2 {
                            self.cycle_attribute >>= 4;
                        }
                        self.cycle_attribute &= 3;
                    }
                    4 => {
                        self.address_bus = ((self.vram_address & 0b0111000000000000) >> 12)
                            | self.cycle_next_character as u16 * 16
                            | (if self.bg_pattern_table { 0x1000 } else { 0 });
                        self.cycle_temp = self.read_ppu(self.address_bus);
                    }
                    5 => {
                        self.pattern_low_bit_plane = self.cycle_temp;
                        self.address_bus = self.address_bus.wrapping_add(8);
                    }
                    6 => {
                        self.cycle_temp = self.read_ppu(self.address_bus);
                    }
                    7 => {
                        self.pattern_high_bit_plane = self.cycle_temp;
                        if (self.vram_address & 0x001f) == 31 {
                            self.vram_address &= 0xffe0;
                            self.vram_address ^= 0x0400;
                        } else {
                            self.vram_address = self.vram_address.wrapping_add(1);
                        }
                    }
                    _ => unreachable!(),
                };
            }

            if self.ppu_dot == 256 {
                self.increment_y_scroll();
            } else if self.ppu_dot == 257 {
                self.reset_x_scroll();
            }

            if self.ppu_dot >= 280 && self.ppu_dot <= 304 && self.ppu_scanline == 261 {
                self.reset_y_scroll();
            }
        }

        // Visible area: scanlines 0-239, dots 1-256.
        if self.ppu_scanline < 240 && self.ppu_dot > 0 && self.ppu_dot <= 256 {
            let x = (self.ppu_dot - 1) as usize;
            let y = self.ppu_scanline as usize;

            let mut palette_high = 0;
            let mut palette_low = 0;

            // Determine colour bits from shift registers
            if self.mask_render_background && (self.ppu_dot > 8 || self.mask_background_8px) {
                let col_0 =
                    ((self.shift_register_pattern_l >> (15 - self.ppu_scroll_fine_x)) & 1) as u8;
                let col_1 =
                    ((self.shift_register_pattern_h >> (15 - self.ppu_scroll_fine_x)) & 1) as u8;
                palette_low = (col_1 << 1) | col_0;

                let pal_0 = (((self.shift_register_attribute_l) >> (15 - self.ppu_scroll_fine_x))
                    & 1) as u8;
                let pal_1 = (((self.shift_register_attribute_h) >> (15 - self.ppu_scroll_fine_x))
                    & 1) as u8;
                palette_high = (pal_1 << 1) | pal_0;

                // Map PPU pixels to Palette RAM indicies
                let palette_ram_addr = if palette_low == 0 {
                    0
                } else {
                    (palette_high << 2) | palette_low
                } as usize;

                // Look up NES colour index from PPU Palette RAM
                let nes_colour_index = (self.palette_ram[palette_ram_addr] & 0x3f) as usize;

                // Look up the 0xRRGGBB value
                self.frame_buffer[y * 256 + x] = NES_PALETTE[nes_colour_index];
            }
        }

        // Signal a frame is done at the start of VBlank
        if self.ppu_dot == 1 && self.ppu_scanline == 241 {
            self.v_blank = true;
            self.frame_complete = true;
        } else if self.ppu_dot == 1 && self.ppu_scanline == 261 {
            self.v_blank = false;
        }

        self.ppu_dot += 1;
        if self.ppu_dot >= 341 {
            self.ppu_dot = 0;
            self.ppu_scanline += 1;
            if self.ppu_scanline >= 262 {
                self.ppu_scanline = 0;
            }
        }
    }
}
