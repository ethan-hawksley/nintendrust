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
    scanline: u16,
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
    oam_address: u8,
    sprite_evaluation_tick: u8,
    status_overflow: bool,
    status_sprite_zero_hit: bool,
    scanline_contains_sprite_zero: bool,
    sprite_zero_on_next_scanline: bool,
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
            scanline: 0,
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
            sprite_zero_on_next_scanline: false,
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
            0x2002 => {
                0 | (self.v_blank as u8) << 7
                    | (self.status_sprite_zero_hit as u8) << 6
                    | (self.status_overflow as u8) << 5
            }
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
                let status = 0
                    | (self.v_blank as u8) << 7
                    | (self.status_sprite_zero_hit as u8) << 6
                    | (self.status_overflow as u8) << 5;

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
            0x2003 => {
                // OAM ADDR
                self.oam_address = value;
            }
            0x2004 => {
                // OAM DATA
                self.oam[self.oam_address as usize] = value;
                self.oam_address = self.oam_address.wrapping_add(1);
            }
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
                    let idx = (self.vram_address & 0x0F) as usize;
                    self.palette_ram[idx] = value;
                } else {
                    let idx = (self.vram_address & 0x1F) as usize;
                    self.palette_ram[idx] = value;
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

    fn get_sprite_pattern_address(&self, sprite_index: usize) -> u16 {
        if !self.use_8x16_sprites {
            // 8x8 sprites.
            if ((self.sprite_attribute[sprite_index & 0x07] >> 7) & 1) == 0 {
                // Attributes are not set to flip Y.
                (if self.sprite_pattern_table {
                    0x1000u16
                } else {
                    0u16
                })
                .wrapping_add((self.sprite_pattern[sprite_index & 0x07] as u16) << 4)
                .wrapping_add(
                    self.scanline
                        .wrapping_sub(self.sprite_y_position[sprite_index & 0x07] as u16),
                )
            } else {
                // Attributes are set to flip Y.
                (if self.sprite_pattern_table {
                    0x1000u16
                } else {
                    0u16
                })
                .wrapping_add((self.sprite_pattern[sprite_index & 0x07] as u16) << 4)
                .wrapping_add(
                    7 - (self
                        .scanline
                        .wrapping_sub(self.sprite_y_position[sprite_index & 0x07] as u16)
                        & 7),
                )
            }
        } else {
            // 8x16 sprites.
            let sprite_pat = self.sprite_pattern[sprite_index];
            let sprite_attr = self.sprite_attribute[sprite_index];
            let sprite_y = self.sprite_y_position[sprite_index];

            let bank = if (sprite_pat & 1) == 1 {
                0x1000
            } else {
                0x0000
            };
            let tile_addr = (sprite_pat & 0xFE) as u16;
            let diff = self.scanline.wrapping_sub(sprite_y as u16);

            if ((sprite_attr >> 7) & 1) == 0 {
                // Attributes are not set to flip Y
                if diff < 8 {
                    bank | (tile_addr << 4) | diff
                } else {
                    bank | (tile_addr << 4) | 16 | (diff & 7)
                }
            } else {
                // Attributes are set to flip Y
                if diff < 8 {
                    bank | (tile_addr << 4) | 16 | ((7_u16.wrapping_sub(diff)) & 7)
                } else {
                    bank | (tile_addr << 4) | (7_u16.wrapping_sub(diff & 7))
                }
            }
        }
    }

    fn sprite_evaluation(&mut self) {
        if self.ppu_dot == 0 {
            self.secondary_oam_address = 0;
            self.secondary_oam_full = false;
            self.sprite_evaluation_oam_overflowed = false;
        } else if self.ppu_dot > 0 && self.ppu_dot <= 64 {
            if (self.ppu_dot & 1) == 1 {
                // Odd PPU cycles load the value $FF
                self.sprite_evaluation_temp = 0xff;
            } else {
                // Even PPU cycles store the value in Secondary OAM
                self.secondary_oam[(self.secondary_oam_address as usize) & 0x1f] =
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
                if !self.sprite_evaluation_oam_overflowed {
                    if !self.secondary_oam_full && self.secondary_oam_address < 32 {
                        // If secondary OAM isn't full, the write always occurs.
                        self.secondary_oam[self.secondary_oam_address as usize] =
                            self.sprite_evaluation_temp;
                    }
                }
                // Even PPU cycles store the value in secondary OAM.
                if self.sprite_evaluation_tick == 0 {
                    // Reading index 0 of an object's set of four bytes.
                    if self.scanline >= self.sprite_evaluation_temp as u16
                        && (self.scanline - self.sprite_evaluation_temp as u16)
                            < (if self.use_8x16_sprites { 16 } else { 8 })
                    {
                        // This object is on the scanline.
                        if !self.secondary_oam_full {
                            // Increment for next write to secondary OAM.
                            if self.secondary_oam_address < 32 {
                                self.secondary_oam_address += 1;
                            }
                            // Increment for next read from OAM.
                            self.oam_address += 1;
                            if self.ppu_dot == 66 {
                                // Index 0 is evaluated on PPU dot 66
                                self.sprite_zero_on_next_scanline = true;
                            }
                        } else {
                            self.status_overflow = true;
                        }
                        self.sprite_evaluation_tick += 1;
                    } else {
                        self.oam_address = self.oam_address.wrapping_add(4);
                    }
                } else {
                    // Reading index 1, 2, or 3 of an object's OAM data.
                    // Increment for next write to secondary OAM
                    if self.secondary_oam_address < 32 {
                        self.secondary_oam_address += 1;
                    }
                    // Increment for next read from OAM
                    self.oam_address = self.oam_address.wrapping_add(1);

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
                self.scanline_contains_sprite_zero = self.sprite_zero_on_next_scanline;
                self.sprite_zero_on_next_scanline = false;
            }
            match self.sprite_evaluation_tick {
                0 => {
                    // Set this object's Y position in the array.
                    self.sprite_y_position[((self.secondary_oam_address / 4) as usize) & 0x07] =
                        self.secondary_oam[(self.secondary_oam_address as usize) & 0x1f];
                    self.secondary_oam_address += 1;
                }
                1 => {
                    // Set this object's pattern in the array.
                    self.sprite_pattern[((self.secondary_oam_address / 4) as usize) & 0x07] =
                        self.secondary_oam[(self.secondary_oam_address as usize) & 0x1f];
                    self.secondary_oam_address += 1;
                }
                2 => {
                    // Set this object's attributes in the array.
                    self.sprite_attribute[((self.secondary_oam_address / 4) as usize) & 0x07] =
                        self.secondary_oam[(self.secondary_oam_address as usize) & 0x1f];
                    self.secondary_oam_address += 1;
                }
                3 => {
                    // Set this object's X position in the array.
                    self.sprite_x_position[((self.secondary_oam_address / 4) as usize) & 0x07] =
                        self.secondary_oam[(self.secondary_oam_address as usize) & 0x1f];
                }
                4 => {
                    let sprite_index = (self.secondary_oam_address / 4) as usize;
                    self.address_bus = self.get_sprite_pattern_address(sprite_index);
                }
                5 => {
                    self.sprite_evaluation_temp = self.read_ppu(self.address_bus);
                    if self.scanline == 261 {
                        // Clear if this is the pre-render line.
                        self.sprite_evaluation_temp = 0;
                    }
                    if ((self.sprite_attribute[((self.secondary_oam_address / 4) as usize) & 0x07]
                        >> 6)
                        & 1)
                        == 1
                    {
                        // Attributes are set up to flip X.
                        self.sprite_evaluation_temp = self.sprite_evaluation_temp.reverse_bits();
                    }
                    self.sprite_shift_register_l
                        [((self.secondary_oam_address / 4) as usize) & 0x07] =
                        self.sprite_evaluation_temp;
                }
                6 => {
                    self.address_bus += 8;
                }
                7 => {
                    self.sprite_evaluation_temp = self.read_ppu(self.address_bus);
                    if self.scanline == 261 {
                        // Clear if this is the pre-render line.
                        self.sprite_evaluation_temp = 0;
                    }
                    if ((self.sprite_attribute[((self.secondary_oam_address / 4) as usize) & 0x07]
                        >> 6)
                        & 1)
                        == 1
                    {
                        // Attributes are set up to flip X.
                        self.sprite_evaluation_temp = self.sprite_evaluation_temp.reverse_bits();
                    }
                    self.sprite_shift_register_h
                        [((self.secondary_oam_address / 4) as usize) & 0x07] =
                        self.sprite_evaluation_temp;
                    self.secondary_oam_address += 1;
                }
                _ => unreachable!(),
            }
            self.sprite_evaluation_tick += 1;
            // Reset tick at 8.
            self.sprite_evaluation_tick &= 7;
        }
    }

    pub fn emulate_ppu(&mut self) {
        // Signal a frame is done at the start of VBlank
        if self.ppu_dot == 1 && self.scanline == 241 {
            self.v_blank = true;
            self.frame_complete = true;
        } else if self.ppu_dot == 1 && self.scanline == 261 {
            self.v_blank = false;
            self.status_overflow = false;
            self.status_sprite_zero_hit = false;
        }

        let rendering_enabled = self.mask_render_background || self.mask_render_sprites;
        let visible_or_prerender = self.scanline < 240 || self.scanline == 261;
        let fetching_dot = (self.ppu_dot > 0 && self.ppu_dot <= 256)
            || (self.ppu_dot > 320 && self.ppu_dot <= 336);

        if visible_or_prerender && rendering_enabled {
            self.sprite_evaluation();

            if fetching_dot {
                if self.mask_render_background {
                    self.shift_register_pattern_l <<= 1;
                    self.shift_register_pattern_h <<= 1;
                    self.shift_register_attribute_l <<= 1;
                    self.shift_register_attribute_h <<= 1;
                }

                // Sprite X countdown / shift is done after pixel drawing (see below).

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

            if self.ppu_dot >= 280 && self.ppu_dot <= 304 && self.scanline == 261 {
                self.reset_y_scroll();
            }
        }

        // Visible area: scanlines 0-239, dots 1-256.
        if self.scanline < 240 && self.ppu_dot > 0 && self.ppu_dot <= 256 {
            let x = (self.ppu_dot - 1) as usize;
            let y = self.scanline as usize;

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
                if palette_low == 0 {
                    palette_high = 0;
                }
            }
            // Which colour palette to use.
            let mut sprite_palette_high = 0;
            // Index into colour palette
            let mut sprite_palette_low = 0;
            let mut sprite_priority = false;

            if self.mask_render_sprites && (self.ppu_dot > 8 || self.mask_sprites_8px) {
                for i in 0..8 {
                    if self.sprite_x_position[i] == 0 && i < (self.secondary_oam_size / 4) as usize
                    {
                        // Take bit for pattern low bit plane.
                        let sprite_pixel_l = ((self.sprite_shift_register_l[i]) & 0x80) != 0;
                        // Take bit for pattern high bit plane.
                        let sprite_pixel_h = ((self.sprite_shift_register_h[i]) & 0x80) != 0;
                        let pixel_value = 0
                            | if sprite_pixel_l { 1 } else { 0 }
                            | if sprite_pixel_h { 2 } else { 0 };

                        // Only use this sprite's data if the pixel is opaque
                        if pixel_value != 0 {
                            sprite_palette_low = pixel_value;
                            // Read palette from secondary OAM attributes.
                            sprite_palette_high = (self.sprite_attribute[i] & 0x03) | 0x04;
                            // Read priority from secondary OAM attributes.
                            sprite_priority = ((self.sprite_attribute[i] >> 5) & 1) == 0;

                            if i == 0
                                && self.scanline_contains_sprite_zero
                                && palette_low != 0
                                && self.mask_render_background
                                && self.ppu_dot < 256
                            {
                                self.status_sprite_zero_hit = true;
                            }
                            break;
                        }
                    } else {
                        continue;
                    }
                }
            }

            if (sprite_priority && palette_low != 0) || palette_low == 0 {
                palette_low = sprite_palette_low;
                palette_high = sprite_palette_high;
                if palette_low == 0 {
                    palette_high = 0;
                }
            }

            let palette_ram_addr = ((palette_high << 2) | palette_low) as usize;

            // Look up NES colour index from PPU Palette RAM
            let nes_colour_index = (self.palette_ram[palette_ram_addr] & 0x3f) as usize;

            // Look up the 0xRRGGBB value
            self.frame_buffer[y * 256 + x] = NES_PALETTE[nes_colour_index];
        }

        // Sprite X countdown and shift registers - done AFTER pixel drawing so
        // the current shift register state is read before being advanced.
        if self.mask_render_sprites
            && self.scanline < 240
            && self.ppu_dot >= 1
            && self.ppu_dot <= 256
        {
            for i in 0..8 {
                if self.sprite_x_position[i] > 0 {
                    self.sprite_x_position[i] -= 1;
                } else {
                    self.sprite_shift_register_l[i] <<= 1;
                    self.sprite_shift_register_h[i] <<= 1;
                }
            }
        }

        self.ppu_dot += 1;
        if self.ppu_dot >= 341 {
            self.ppu_dot = 0;
            self.scanline += 1;
            if self.scanline >= 262 {
                self.scanline = 0;
            }
        }
    }
}
