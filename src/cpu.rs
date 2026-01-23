use crate::bus::Bus;

pub struct Cpu {
    program_counter: u16,
    a: u8,
    x: u8,
    y: u8,
    stack_pointer: u8,
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
            stack_pointer: 0,
            halted: false,
            flag_carry: false,
            flag_overflow: false,
            flag_negative: false,
            flag_zero: false,
            flag_decimal: false,
            flag_interrupt_disable: false,
        }
    }

    pub fn reset(&mut self, bus: &Bus) {
        let pc_low = bus.read(0xFFFC);
        let pc_high = bus.read(0xFFFD);
        self.program_counter = (pc_high as u16 * 0x100) + pc_low as u16;
        self.flag_interrupt_disable = true;
        self.stack_pointer = 0xFD
    }

    fn push(&mut self, bus: &mut Bus, value: u8) {
        // Store to the stack, and decrement stack pointer
        bus.write(0x100 + self.stack_pointer as u16, value);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    fn pull(&mut self, bus: &Bus) -> u8 {
        // Increment the stack pointer, and read from the stack
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        let value = bus.read(0x100 + self.stack_pointer as u16);
        value
    }

    fn get_status_register(&self, flag_break: bool) -> u8 {
        (self.flag_carry as u8)
            | (self.flag_zero as u8) << 1
            | (self.flag_interrupt_disable as u8) << 2
            | (self.flag_decimal as u8) << 3
            | (flag_break as u8) << 4
            | 1 << 5 // Bit 5 is always set to 1
            | (self.flag_overflow as u8) << 6
            | (self.flag_negative as u8) << 7
    }

    fn set_status_register(&mut self, data: u8) {
        self.flag_carry = (data & 0x01) != 0;
        self.flag_zero = (data & 0x02) != 0;
        self.flag_interrupt_disable = (data & 0x04) != 0;
        self.flag_decimal = (data & 0x08) != 0;
        self.flag_overflow = (data & 0x40) != 0;
        self.flag_negative = (data & 0x80) != 0;
    }

    fn read_and_increment(&mut self, bus: &Bus) -> u8 {
        let value = bus.read(self.program_counter);
        self.program_counter += 1;
        value
    }

    fn read_absolute_addressed(&mut self, bus: &Bus) -> u16 {
        let value_low = bus.read(self.program_counter);
        self.program_counter += 1;
        let value_high = bus.read(self.program_counter);
        self.program_counter += 1;
        value_high as u16 * 0x100 + value_low as u16
    }

    fn shift_left(&mut self, bus: &mut Bus, address: u16) {
        let mut value = bus.read(address);
        self.flag_carry = value & 0x80 != 0;
        value <<= 1;
        self.flag_zero = value == 0;
        self.flag_negative = value & 0x80 != 0;
        bus.write(address, value);
    }

    fn shift_right(&mut self, bus: &mut Bus, address: u16) {
        let mut value = bus.read(address);
        self.flag_carry = value & 1 != 0;
        value >>= 1;
        self.flag_zero = value == 0;
        self.flag_negative = value & 0x80 != 0;
        bus.write(address, value);
    }

    fn rotate_left(&mut self, bus: &mut Bus, address: u16) {
        let mut value = bus.read(address);
        let old_carry = self.flag_carry;
        self.flag_carry = value & 0x80 != 0;
        value <<= 1;
        if old_carry {
            value |= 1;
        }
        self.flag_zero = value == 0;
        self.flag_negative = value & 0x80 != 0;
        bus.write(address, value);
    }

    fn rotate_right(&mut self, bus: &mut Bus, address: u16) {
        let mut value = bus.read(address);
        let old_carry = self.flag_carry;
        self.flag_carry = value & 1 != 0;
        value >>= 1;
        if old_carry {
            value |= 0x80;
        }
        self.flag_zero = value == 0;
        self.flag_negative = value & 0x80 != 0;
        bus.write(address, value);
    }

    fn increment(&mut self, bus: &mut Bus, address: u16) {
        let mut value = bus.read(address);
        value = value.wrapping_add(1);
        self.flag_zero = value == 0;
        self.flag_negative = value & 0x80 != 0;
        bus.write(address, value);
    }

    fn decrement(&mut self, bus: &mut Bus, address: u16) {
        let mut value = bus.read(address);
        value = value.wrapping_sub(1);
        self.flag_zero = value == 0;
        self.flag_negative = value & 0x80 != 0;
        bus.write(address, value);
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
            0x06 => {
                // ASL Zero Page
                let address = self.read_and_increment(bus);
                self.shift_left(bus, address as u16);
                cycles = 5;
            }
            0x08 => {
                // PHP
                let status = self.get_status_register(true);
                self.push(bus, status);
                cycles = 3;
            }
            0x0A => {
                // ASL A
                self.flag_carry = self.a & 0x80 != 0;
                self.a <<= 1;
                self.flag_zero = self.a == 0;
                self.flag_negative = self.a & 0x80 != 0;
                cycles = 2;
            }
            0x0E => {
                // ASL Absolute
                let address = self.read_absolute_addressed(bus);
                self.shift_left(bus, address);
                cycles = 6;
            }
            0x10 => {
                // BPL
                let destination_offset = self.read_and_increment(bus);
                if !self.flag_negative {
                    let mut signed_offset = destination_offset as i16;
                    if signed_offset & 0x80 != 0 {
                        signed_offset -= 256;
                    }
                    let new_program_counter = self.program_counter + signed_offset as u16;
                    if new_program_counter & 0xff00 == self.program_counter & 0xff00 {
                        cycles = 3;
                    } else {
                        cycles = 4;
                    }
                    self.program_counter = new_program_counter;
                } else {
                    cycles = 2;
                }
            }
            0x18 => {
                // CLC
                self.flag_carry = false;
                cycles = 2;
            }
            0x20 => {
                // JSR
                let destination_address_low = bus.read(self.program_counter);
                self.program_counter += 1;
                let destination_address_high = bus.read(self.program_counter);
                self.push(bus, (self.program_counter / 256) as u8);
                self.push(bus, self.program_counter as u8);
                self.program_counter =
                    destination_address_high as u16 * 256 + destination_address_low as u16;
                cycles = 6;
            }
            0x26 => {
                // ROL Zero Page
                let address = self.read_and_increment(bus);
                self.rotate_left(bus, address as u16);
                cycles = 5;
            }
            0x28 => {
                // PLP
                let status = self.pull(bus);
                self.set_status_register(status);
                cycles = 4;
            }
            0x2A => {
                // ROL A
                let old_carry = self.flag_carry;
                self.flag_carry = self.a & 0x80 != 0;
                self.a <<= 1;
                if old_carry {
                    self.a |= 1;
                }
                self.flag_zero = self.a == 0;
                self.flag_negative = self.a & 0x80 != 0;
                cycles = 2;
            }
            0x2E => {
                // ROL Absolute
                let address = self.read_absolute_addressed(bus);
                self.rotate_left(bus, address);
                cycles = 6;
            }
            0x30 => {
                // BMI
                let destination_offset = self.read_and_increment(bus);
                if self.flag_negative {
                    let mut signed_offset = destination_offset as i16;
                    if signed_offset & 0x80 != 0 {
                        signed_offset -= 256;
                    }
                    let new_program_counter = self.program_counter + signed_offset as u16;
                    if new_program_counter & 0xff00 == self.program_counter & 0xff00 {
                        cycles = 3;
                    } else {
                        cycles = 4;
                    }
                    self.program_counter = new_program_counter;
                } else {
                    cycles = 2;
                }
            }
            0x38 => {
                // SEC
                self.flag_carry = true;
                cycles = 2;
            }
            0x46 => {
                // LSR Zero Page
                let address = self.read_and_increment(bus);
                self.shift_right(bus, address as u16);
                cycles = 5;
            }
            0x48 => {
                // PHA
                self.push(bus, self.a);
                cycles = 3;
            }
            0x4A => {
                // LSR A
                self.flag_carry = self.a & 1 != 0;
                self.a >>= 1;
                self.flag_zero = self.a == 0;
                self.flag_negative = self.a & 0x80 != 0;
                cycles = 2;
            }
            0x4C => {
                // JMP
                self.program_counter = self.read_absolute_addressed(bus);
                cycles = 3;
            }
            0x4E => {
                // LSR Absolute
                let address = self.read_absolute_addressed(bus);
                self.shift_right(bus, address);
                cycles = 6;
            }
            0x50 => {
                // BVC
                let destination_offset = self.read_and_increment(bus);
                if !self.flag_overflow {
                    let mut signed_offset = destination_offset as i16;
                    if signed_offset & 0x80 != 0 {
                        signed_offset -= 256;
                    }
                    let new_program_counter = self.program_counter + signed_offset as u16;
                    if new_program_counter & 0xff00 == self.program_counter & 0xff00 {
                        cycles = 3;
                    } else {
                        cycles = 4;
                    }
                    self.program_counter = new_program_counter;
                } else {
                    cycles = 2;
                }
            }
            0x58 => {
                // CLI
                self.flag_interrupt_disable = false;
                cycles = 2;
            }
            0x60 => {
                // RTS
                let return_address_low = self.pull(bus);
                let return_address_high = self.pull(bus);
                self.program_counter = return_address_high as u16 * 256 + return_address_low as u16;
                self.program_counter += 1;
                cycles = 6;
            }
            0x66 => {
                // ROR Zero Page
                let address = self.read_and_increment(bus);
                self.rotate_right(bus, address as u16);
                cycles = 5;
            }
            0x68 => {
                // PLA
                self.a = self.pull(bus);
                self.flag_zero = self.a == 0;
                self.flag_negative = self.a >= 0x80;
                cycles = 4;
            }
            0x6A => {
                // ROR A
                let old_carry = self.flag_carry;
                self.flag_carry = self.a & 1 != 0;
                self.a >>= 1;
                if old_carry {
                    self.a |= 0x80;
                }
                self.flag_zero = self.a == 0;
                self.flag_negative = self.a & 0x80 != 0;
                cycles = 2;
            }
            0x6E => {
                // ROR Absolute
                let address = self.read_absolute_addressed(bus);
                self.rotate_right(bus, address);
                cycles = 6;
            }
            0x70 => {
                // BVS
                let destination_offset = self.read_and_increment(bus);
                if self.flag_overflow {
                    let mut signed_offset = destination_offset as i16;
                    if signed_offset & 0x80 != 0 {
                        signed_offset -= 256;
                    }
                    let new_program_counter = self.program_counter + signed_offset as u16;
                    if new_program_counter & 0xff00 == self.program_counter & 0xff00 {
                        cycles = 3;
                    } else {
                        cycles = 4;
                    }
                    self.program_counter = new_program_counter;
                } else {
                    cycles = 2;
                }
            }
            0x78 => {
                // SEI
                self.flag_interrupt_disable = true;
                cycles = 2;
            }
            0x84 => {
                // STY Zero Page
                let destination_address = self.read_and_increment(bus);
                bus.write(destination_address as u16, self.y);
                cycles = 3;
            }
            0x85 => {
                // STA Zero Page
                let destination_address = self.read_and_increment(bus);
                bus.write(destination_address as u16, self.a);
                cycles = 3;
            }
            0x86 => {
                // STX Zero Page
                let destination_address = self.read_and_increment(bus);
                bus.write(destination_address as u16, self.x);
                cycles = 3;
            }
            0x88 => {
                // DEY
                self.y = self.y.wrapping_sub(1);
                self.flag_zero = self.y == 0;
                self.flag_negative = self.y & 0x80 != 0;
                cycles = 2;
            }
            0x8A => {
                // TXA
                self.a = self.x;
                self.flag_zero = self.a == 0;
                self.flag_negative = self.a & 0x80 != 0;
                cycles = 2;
            }
            0x8C => {
                // STY Absolute
                let destination_address = self.read_absolute_addressed(bus);
                bus.write(destination_address, self.y);
                cycles = 4;
            }
            0x8D => {
                // STA Absolute
                let destination_address = self.read_absolute_addressed(bus);
                bus.write(destination_address, self.a);
                cycles = 4;
            }
            0x8E => {
                // STX Absolute
                let destination_address = self.read_absolute_addressed(bus);
                bus.write(destination_address, self.x);
                cycles = 4;
            }
            0x90 => {
                // BCC
                let destination_offset = self.read_and_increment(bus);
                if !self.flag_carry {
                    let mut signed_offset = destination_offset as i16;
                    if signed_offset & 0x80 != 0 {
                        signed_offset -= 256;
                    }
                    let new_program_counter = self.program_counter + signed_offset as u16;
                    if new_program_counter & 0xff00 == self.program_counter & 0xff00 {
                        cycles = 3;
                    } else {
                        cycles = 4;
                    }
                    self.program_counter = new_program_counter;
                } else {
                    cycles = 2;
                }
            }
            0x98 => {
                // TYA
                self.a = self.y;
                self.flag_zero = self.a == 0;
                self.flag_negative = self.a & 0x80 != 0;
                cycles = 2;
            }
            0x9A => {
                // TXS
                self.stack_pointer = self.x;
                cycles = 2;
            }
            0xA0 => {
                // LDY Immediate
                self.y = self.read_and_increment(bus);
                self.flag_zero = self.y == 0;
                self.flag_negative = self.y & 0x80 != 0;
                cycles = 2;
            }
            0xA2 => {
                // LDX Immediate
                self.x = self.read_and_increment(bus);
                self.flag_zero = self.x == 0;
                self.flag_negative = self.x & 0x80 != 0;
                cycles = 2;
            }
            0xA5 => {
                // LDA Zero Page
                let destination_address = self.read_and_increment(bus);
                self.a = bus.read(destination_address as u16);
                self.flag_zero = self.a == 0;
                self.flag_negative = self.a & 0x80 != 0;
                cycles = 3;
            }
            0xA8 => {
                // TAY
                self.y = self.a;
                self.flag_zero = self.y == 0;
                self.flag_negative = self.y & 0x80 != 0;
                cycles = 2;
            }
            0xAA => {
                // TAX
                self.x = self.a;
                self.flag_zero = self.x == 0;
                self.flag_negative = self.x & 0x80 != 0;
                cycles = 2;
            }
            0xAD => {
                // LDA Absolute
                let destination_address = self.read_absolute_addressed(bus);
                self.a = bus.read(destination_address);
                self.flag_zero = self.a == 0;
                self.flag_negative = self.a & 0x80 != 0;
                cycles = 4;
            }
            0xA9 => {
                // LDA Immediate
                self.a = self.read_and_increment(bus);
                self.flag_zero = self.a == 0;
                self.flag_negative = self.a & 0x80 != 0;
                cycles = 2;
            }
            0xB0 => {
                // BCS
                let destination_offset = self.read_and_increment(bus);
                if self.flag_carry {
                    let mut signed_offset = destination_offset as i16;
                    if signed_offset & 0x80 != 0 {
                        signed_offset -= 256;
                    }
                    let new_program_counter = self.program_counter + signed_offset as u16;
                    if new_program_counter & 0xff00 == self.program_counter & 0xff00 {
                        cycles = 3;
                    } else {
                        cycles = 4;
                    }
                    self.program_counter = new_program_counter;
                } else {
                    cycles = 2;
                }
            }
            0xB8 => {
                // CLV
                self.flag_overflow = false;
                cycles = 2;
            }
            0xBA => {
                // TSX
                self.x = self.stack_pointer;
                self.flag_zero = self.x == 0;
                self.flag_negative = self.x & 0x80 != 0;
                cycles = 2;
            }
            0xC6 => {
                // DEC Zero Page
                let address = self.read_and_increment(bus);
                self.decrement(bus, address as u16);
                cycles = 5;
            }
            0xC8 => {
                // INY
                self.y = self.y.wrapping_add(1);
                self.flag_zero = self.y == 0;
                self.flag_negative = self.y & 0x80 != 0;
                cycles = 2;
            }
            0xCA => {
                // DEX
                self.x = self.x.wrapping_sub(1);
                self.flag_zero = self.x == 0;
                self.flag_negative = self.x & 0x80 != 0;
                cycles = 2;
            }
            0xCE => {
                // DEC Absolute
                let address = self.read_absolute_addressed(bus);
                self.decrement(bus, address);
                cycles = 6;
            }
            0xD0 => {
                // BNE
                let destination_offset = self.read_and_increment(bus);
                if !self.flag_zero {
                    let mut signed_offset = destination_offset as i16;
                    if signed_offset & 0x80 != 0 {
                        signed_offset -= 256;
                    }
                    let new_program_counter = self.program_counter + signed_offset as u16;
                    if new_program_counter & 0xff00 == self.program_counter & 0xff00 {
                        cycles = 3;
                    } else {
                        cycles = 4;
                    }
                    self.program_counter = new_program_counter;
                } else {
                    cycles = 2;
                }
            }
            0xD8 => {
                // CLD
                self.flag_decimal = false;
                cycles = 2;
            }
            0xE6 => {
                // INC Zero Page
                let address = self.read_and_increment(bus);
                self.increment(bus, address as u16);
                cycles = 5;
            }
            0xE8 => {
                // INX
                self.x = self.x.wrapping_add(1);
                self.flag_zero = self.x == 0;
                self.flag_negative = self.x & 0x80 != 0;
                cycles = 2;
            }
            0xEA => {
                // NOP
                cycles = 2;
            }
            0xEE => {
                // INC Absolute
                let address = self.read_absolute_addressed(bus);
                self.increment(bus, address);
                cycles = 6;
            }
            0xF0 => {
                // BEQ
                let destination_offset = self.read_and_increment(bus);
                if self.flag_zero {
                    let mut signed_offset = destination_offset as i16;
                    if signed_offset & 0x80 != 0 {
                        signed_offset -= 256;
                    }
                    let new_program_counter = self.program_counter + signed_offset as u16;
                    if new_program_counter & 0xff00 == self.program_counter & 0xff00 {
                        cycles = 3;
                    } else {
                        cycles = 4;
                    }
                    self.program_counter = new_program_counter;
                } else {
                    cycles = 2;
                }
            }
            0xF8 => {
                // SED
                self.flag_decimal = true;
                cycles = 2;
            }
            _ => {
                // Unknown opcode
                panic!("Unknown opcode {}", opcode)
            }
        }
        println!("{} cycles used", cycles)
    }
}
