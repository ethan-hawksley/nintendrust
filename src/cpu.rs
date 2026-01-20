use crate::bus::Bus;

pub struct Cpu {
    program_counter: u16,
    a: u8,
    x: u8,
    y: u8,
    flag_carry: bool,
    flag_zero: bool,
    flag_interrupt_disable: bool,
    flag_decimal: bool,
    flag_overflow: bool,
    flag_negative: bool,
    pub halted: bool,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            program_counter: 0,
            a: 0,
            x: 0,
            y: 0,
            halted: false,
            flag_carry: false,
            flag_overflow: false,
            flag_negative: false,
            flag_zero: false,
            flag_decimal: false,
            flag_interrupt_disable: false,
        }
    }

    pub fn reset(&mut self, bus: &mut Bus) {
        let pc_low = bus.read(0xFFFC);
        let pc_high = bus.read(0xFFFD);
        self.program_counter = (pc_high as u16 * 0x100) + pc_low as u16;
        self.flag_interrupt_disable = true;
    }

    pub fn emulate_cpu(&mut self, bus: &mut Bus) {
        let opcode = bus.read(self.program_counter);
        println!("0x{:02x}", opcode);
        self.program_counter += 1;
        let mut cycles = 0;

        match opcode {
            0x02 => {
                // HTL
                self.halted = true;
            }
            0x10 => {
                // BPL
                let destination_offset = bus.read(self.program_counter);
                self.program_counter += 1;
                if !self.flag_negative {
                    let mut signed_offset = destination_offset as i16;
                    if signed_offset > 127 {
                        signed_offset -= 256;
                    }
                    let new_program_counter = self.program_counter + signed_offset as u16;
                    if new_program_counter & 0xff00 == self.program_counter & 0xff00 {
                        cycles = 4;
                    } else {
                        cycles = 3;
                    }
                    self.program_counter = new_program_counter;
                } else {
                    cycles = 2;
                }
            }
            0x30 => {
                // BMI
                let destination_offset = bus.read(self.program_counter);
                self.program_counter += 1;
                if self.flag_negative {
                    let mut signed_offset = destination_offset as i16;
                    if signed_offset > 127 {
                        signed_offset -= 256;
                    }
                    let new_program_counter = self.program_counter + signed_offset as u16;
                    if new_program_counter & 0xff00 == self.program_counter & 0xff00 {
                        cycles = 4;
                    } else {
                        cycles = 3;
                    }
                    self.program_counter = new_program_counter;
                } else {
                    cycles = 2;
                }
            }
            0x50 => {
                // BVC
                let destination_offset = bus.read(self.program_counter);
                self.program_counter += 1;
                if !self.flag_overflow {
                    let mut signed_offset = destination_offset as i16;
                    if signed_offset > 127 {
                        signed_offset -= 256;
                    }
                    let new_program_counter = self.program_counter + signed_offset as u16;
                    if new_program_counter & 0xff00 == self.program_counter & 0xff00 {
                        cycles = 4;
                    } else {
                        cycles = 3;
                    }
                    self.program_counter = new_program_counter;
                } else {
                    cycles = 2;
                }
            }
            0x70 => {
                // BVS
                let destination_offset = bus.read(self.program_counter);
                self.program_counter += 1;
                if self.flag_overflow {
                    let mut signed_offset = destination_offset as i16;
                    if signed_offset > 127 {
                        signed_offset -= 256;
                    }
                    let new_program_counter = self.program_counter + signed_offset as u16;
                    if new_program_counter & 0xff00 == self.program_counter & 0xff00 {
                        cycles = 4;
                    } else {
                        cycles = 3;
                    }
                    self.program_counter = new_program_counter;
                } else {
                    cycles = 2;
                }
            }
            0x84 => {
                // STY Zero Page
                let destination_address = bus.read(self.program_counter);
                self.program_counter += 1;
                bus.write(destination_address as u16, self.y);
                cycles = 3;
            }
            0x85 => {
                // STA Zero Page
                let destination_address = bus.read(self.program_counter);
                self.program_counter += 1;
                bus.write(destination_address as u16, self.a);
                cycles = 3;
            }
            0x86 => {
                // STX Zero Page
                let destination_address = bus.read(self.program_counter);
                self.program_counter += 1;
                bus.write(destination_address as u16, self.x);
                cycles = 3;
            }
            0x8C => {
                // STY Absolute
                let destination_address_low = bus.read(self.program_counter);
                self.program_counter += 1;
                let destination_address_high = bus.read(self.program_counter);
                self.program_counter += 1;
                bus.write(
                    destination_address_high as u16 * 0x100 + destination_address_low as u16,
                    self.y,
                );
                cycles = 4;
            }
            0x8D => {
                // STA Absolute
                let destination_address_low = bus.read(self.program_counter);
                self.program_counter += 1;
                let destination_address_high = bus.read(self.program_counter);
                self.program_counter += 1;
                bus.write(
                    destination_address_high as u16 * 0x100 + destination_address_low as u16,
                    self.a,
                );
                cycles = 4;
            }
            0x8E => {
                // STX Absolute
                let destination_address_low = bus.read(self.program_counter);
                self.program_counter += 1;
                let destination_address_high = bus.read(self.program_counter);
                self.program_counter += 1;
                bus.write(
                    destination_address_high as u16 * 0x100 + destination_address_low as u16,
                    self.x,
                );
                cycles = 4;
            }
            0x90 => {
                // BCC
                let destination_offset = bus.read(self.program_counter);
                self.program_counter += 1;
                if !self.flag_carry {
                    let mut signed_offset = destination_offset as i16;
                    if signed_offset > 127 {
                        signed_offset -= 256;
                    }
                    let new_program_counter = self.program_counter + signed_offset as u16;
                    if new_program_counter & 0xff00 == self.program_counter & 0xff00 {
                        cycles = 4;
                    } else {
                        cycles = 3;
                    }
                    self.program_counter = new_program_counter;
                } else {
                    cycles = 2;
                }
            }
            0xA0 => {
                // LDY Immediate
                self.y = bus.read(self.program_counter);
                self.program_counter += 1;
                self.flag_zero = self.y == 0;
                self.flag_negative = self.y > 127;
                cycles = 2;
            }
            0xA2 => {
                // LDX Immediate
                self.x = bus.read(self.program_counter);
                self.program_counter += 1;
                self.flag_zero = self.x == 0;
                self.flag_negative = self.x > 127;
                cycles = 2;
            }
            0xA5 => {
                // LDA Zero Page
                let destination_address = bus.read(self.program_counter);
                self.program_counter += 1;
                self.a = bus.read(destination_address as u16);
                self.flag_zero = self.a == 0;
                self.flag_negative = self.a > 127;
                cycles = 3;
            }
            0xAD => {
                // LDA Absolute
                let destination_address_low = bus.read(self.program_counter);
                self.program_counter += 1;
                let destination_address_high = bus.read(self.program_counter);
                self.program_counter += 1;
                self.a = bus
                    .read(destination_address_high as u16 * 0x100 + destination_address_low as u16);
                self.flag_zero = self.a == 0;
                self.flag_negative = self.a > 127;
                cycles = 4;
            }
            0xA9 => {
                // LDA Immediate
                self.a = bus.read(self.program_counter);
                self.program_counter += 1;
                self.flag_zero = self.a == 0;
                self.flag_negative = self.a > 127;
                cycles = 2;
            }
            0xB0 => {
                // BCS
                let destination_offset = bus.read(self.program_counter);
                self.program_counter += 1;
                if self.flag_carry {
                    let mut signed_offset = destination_offset as i16;
                    if signed_offset > 127 {
                        signed_offset -= 256;
                    }
                    let new_program_counter = self.program_counter + signed_offset as u16;
                    if new_program_counter & 0xff00 == self.program_counter & 0xff00 {
                        cycles = 4;
                    } else {
                        cycles = 3;
                    }
                    self.program_counter = new_program_counter;
                } else {
                    cycles = 2;
                }
            }
            0xD0 => {
                // BNE
                let destination_offset = bus.read(self.program_counter);
                self.program_counter += 1;
                if !self.flag_zero {
                    let mut signed_offset = destination_offset as i16;
                    if signed_offset > 127 {
                        signed_offset -= 256;
                    }
                    let new_program_counter = self.program_counter + signed_offset as u16;
                    if new_program_counter & 0xff00 == self.program_counter & 0xff00 {
                        cycles = 4;
                    } else {
                        cycles = 3;
                    }
                    self.program_counter = new_program_counter;
                } else {
                    cycles = 2;
                }
            }
            0xF0 => {
                // BEQ
                let destination_offset = bus.read(self.program_counter);
                self.program_counter += 1;
                if self.flag_zero {
                    let mut signed_offset = destination_offset as i16;
                    if signed_offset > 127 {
                        signed_offset -= 256;
                    }
                    let new_program_counter = self.program_counter + signed_offset as u16;
                    if new_program_counter & 0xff00 == self.program_counter & 0xff00 {
                        cycles = 4;
                    } else {
                        cycles = 3;
                    }
                    self.program_counter = new_program_counter;
                } else {
                    cycles = 2;
                }
            }
            _ => {
                // Unknown opcode
                panic!("Unknown opcode {}", opcode)
            }
        }
        println!("{} cycles used", cycles)
    }
}
