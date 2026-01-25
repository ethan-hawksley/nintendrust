use crate::bus::Bus;
use crate::opcodes::OPCODES;

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

    fn get_operand_mode(&self, opcode: u8) -> u8 {
        match opcode {
            // Single Byte Instructions
            0x00 | 0x08 | 0x18 | 0x28 | 0x38 | 0x40 | 0x48 | 0x58 | 0x60 | 0x68 | 0x78 | 0x88
            | 0x8A | 0x98 | 0x9A | 0xA8 | 0xAA | 0xB8 | 0xBA | 0xC8 | 0xCA | 0xD8 | 0xE8 | 0xEA
            | 0xF8 | 0x0A | 0x2A | 0x4A | 0x6A => 1,

            // 2 Byte Instructions
            0x05 | 0x06 | 0x09 | 0x10 | 0x24 | 0x25 | 0x26 | 0x29 | 0x30 | 0x45 | 0x46 | 0x49
            | 0x50 | 0x65 | 0x66 | 0x69 | 0x70 | 0x84 | 0x85 | 0x86 | 0x90 | 0xA0 | 0xA2 | 0xA5
            | 0xA9 | 0xB0 | 0xC0 | 0xC4 | 0xC5 | 0xC6 | 0xC9 | 0xD0 | 0xE0 | 0xE4 | 0xE5 | 0xE6
            | 0xE9 | 0xF0 => 2,

            // 3 Byte Instructions
            0x0D | 0x0E | 0x20 | 0x2C | 0x2D | 0x2E | 0x4C | 0x4D | 0x4E | 0x6C | 0x6D | 0x6E
            | 0x8C | 0x8D | 0x8E | 0xAC | 0xAD | 0xAE | 0xCC | 0xCD | 0xCE | 0xEC | 0xED | 0xEE => {
                3
            }

            // Any other instructions
            _ => 1,
        }
    }

    pub fn trace(&self, bus: &Bus) -> String {
        let code = bus.read(self.program_counter);
        let length = self.get_operand_mode(code);

        let mut hex_dump = vec![];
        hex_dump.push(code);

        if length > 1 {
            let mem = bus.read(self.program_counter + 1);
            hex_dump.push(mem);
        }
        if length > 2 {
            let mem = bus.read(self.program_counter + 2);
            hex_dump.push(mem);
        }

        let hex_str = hex_dump
            .iter()
            .map(|z| format!("{:02X}", z))
            .collect::<Vec<String>>()
            .join(" ");

        let mnemonic = OPCODES[code as usize];

        let status = self.get_status_register(false) | 0x30; // 0x30 sets bit 4 and 5

        format!(
            "{:04X}  {:8} {: >4} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
            self.program_counter,
            hex_str,
            mnemonic,
            self.a,
            self.x,
            self.y,
            status,
            self.stack_pointer,
        )
    }

    pub fn reset(&mut self, bus: &Bus) {
        let pc_low = bus.read(0xFFFC);
        let pc_high = bus.read(0xFFFD);
        self.program_counter = (pc_high as u16 * 0x100) + pc_low as u16;
        self.flag_interrupt_disable = true;
        self.stack_pointer = 0xFD
    }

    fn push(&mut self, bus: &mut Bus, value: u8) {
        bus.write(0x100 + self.stack_pointer as u16, value);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    fn pull(&mut self, bus: &Bus) -> u8 {
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
            | 1 << 5
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

    fn read_immediate_addressed(&mut self, bus: &Bus) -> u8 {
        let value = bus.read(self.program_counter);
        self.program_counter = self.program_counter.wrapping_add(1);
        value
    }

    fn read_absolute_addressed(&mut self, bus: &Bus) -> u16 {
        let value_low = bus.read(self.program_counter);
        self.program_counter = self.program_counter.wrapping_add(1);
        let value_high = bus.read(self.program_counter);
        self.program_counter = self.program_counter.wrapping_add(1);
        (value_high as u16) << 8 | value_low as u16
    }

    fn read_absolute_addressed_x_indexed(&mut self, bus: &Bus) -> u16 {
        let value_low = bus.read(self.program_counter);
        self.program_counter = self.program_counter.wrapping_add(1);
        let value_high = bus.read(self.program_counter);
        self.program_counter = self.program_counter.wrapping_add(1);
        ((value_high as u16) << 8 | value_low as u16).wrapping_add(self.x as u16)
    }

    fn read_absolute_addressed_y_indexed(&mut self, bus: &Bus) -> u16 {
        let value_low = bus.read(self.program_counter);
        self.program_counter = self.program_counter.wrapping_add(1);
        let value_high = bus.read(self.program_counter);
        self.program_counter = self.program_counter.wrapping_add(1);
        ((value_high as u16) << 8 | value_low as u16).wrapping_add(self.y as u16)
    }

    fn read_zero_page_addressed_x_indexed(&mut self, bus: &Bus) -> u16 {
        let address = bus.read(self.program_counter);
        self.program_counter = self.program_counter.wrapping_add(1);
        address.wrapping_add(self.x) as u16
    }

    fn read_zero_page_addressed_y_indexed(&mut self, bus: &Bus) -> u16 {
        let address = bus.read(self.program_counter);
        self.program_counter = self.program_counter.wrapping_add(1);
        address.wrapping_add(self.y) as u16
    }

    fn read_indirect_addressed(&mut self, bus: &Bus) -> u16 {
        let address_low = bus.read(self.program_counter);
        self.program_counter = self.program_counter.wrapping_add(1);
        let address_high = bus.read(self.program_counter);
        self.program_counter = self.program_counter.wrapping_add(1);
        let address = (address_high as u16) << 8 | address_low as u16;
        let value_low = bus.read(address);
        let high_address = (address & 0xFF00) | ((address.wrapping_add(1)) & 0x00FF);
        let value_high = bus.read(high_address);
        (value_high as u16) << 8 | value_low as u16
    }

    fn read_indirect_addressed_x_indexed(&mut self, bus: &Bus) -> u16 {
        let address = bus.read(self.program_counter).wrapping_add(self.x);
        self.program_counter = self.program_counter.wrapping_add(1);
        let value_low = bus.read(address as u16);
        let value_high = bus.read(address.wrapping_add(1) as u16);
        (value_high as u16) << 8 | value_low as u16
    }

    fn read_indirect_addressed_y_indexed(&mut self, bus: &Bus) -> u16 {
        let address = bus.read(self.program_counter);
        self.program_counter = self.program_counter.wrapping_add(1);
        let value_low = bus.read(address as u16);
        let value_high = bus.read(address.wrapping_add(1) as u16);
        ((value_high as u16) << 8 | value_low as u16).wrapping_add(self.y as u16)
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

    fn bitwise_or(&mut self, bus: &Bus, address: u16) {
        let value = bus.read(address);
        self.a |= value;
        self.flag_zero = self.a == 0;
        self.flag_negative = self.a & 0x80 != 0;
    }

    fn bitwise_and(&mut self, bus: &Bus, address: u16) {
        let value = bus.read(address);
        self.a &= value;
        self.flag_zero = self.a == 0;
        self.flag_negative = self.a & 0x80 != 0;
    }

    fn bitwise_eor(&mut self, bus: &Bus, address: u16) {
        let value = bus.read(address);
        self.a ^= value;
        self.flag_zero = self.a == 0;
        self.flag_negative = self.a & 0x80 != 0;
    }

    fn add_carry(&mut self, value: u8) {
        let sum = self.a as u16 + value as u16 + self.flag_carry as u16;
        self.flag_overflow = (!(self.a ^ value) & (self.a ^ sum as u8) & 0x80) != 0;
        self.flag_carry = sum > 0xFF;
        self.a = sum as u8;
        self.flag_zero = self.a == 0;
        self.flag_negative = self.a & 0x80 != 0;
    }

    fn sub_carry(&mut self, value: u8) {
        let diff = self.a as i16 - value as i16 - (if self.flag_carry { 0 } else { 1 });
        self.flag_overflow = ((self.a as i16 ^ value as i16) & (self.a as i16 ^ diff) & 0x80) != 0;
        self.flag_carry = diff >= 0;
        self.a = diff as u8;
        self.flag_zero = self.a == 0;
        self.flag_negative = self.a & 0x80 != 0;
    }

    fn compare(&mut self, register: u8, value: u8) {
        self.flag_carry = register >= value;
        self.flag_zero = register == value;
        self.flag_negative = register.wrapping_sub(value) & 0x80 != 0;
    }

    fn compare_a(&mut self, value: u8) {
        self.compare(self.a, value);
    }

    fn compare_x(&mut self, value: u8) {
        self.compare(self.x, value);
    }

    fn compare_y(&mut self, value: u8) {
        self.compare(self.y, value);
    }

    fn bit_test(&mut self, value: u8) {
        self.flag_zero = self.a & value == 0;
        self.flag_negative = value & 0x80 != 0;
        self.flag_overflow = value & 0x40 != 0;
    }

    fn branch(&mut self, condition: bool, offset: u8) -> u8 {
        if condition {
            let signed_offset = offset as i8 as i16;
            let new_program_counter = self.program_counter.wrapping_add(signed_offset as u16);
            let cycles = if new_program_counter & 0xFF00 == self.program_counter & 0xFF00 {
                3
            } else {
                4
            };
            self.program_counter = new_program_counter;
            cycles
        } else {
            2
        }
    }

    pub fn emulate_cpu(&mut self, bus: &mut Bus) {
        let opcode = bus.read(self.program_counter);
        self.program_counter = self.program_counter.wrapping_add(1);
        let mut _cycles = 0;

        match opcode {
            0x00 => {
                // BRK
                self.program_counter = self.program_counter.wrapping_add(1);
                self.push(bus, (self.program_counter >> 8) as u8);
                self.push(bus, self.program_counter as u8);

                let status = self.get_status_register(true);
                self.push(bus, status);

                self.flag_interrupt_disable = true;

                let destination_address_low = bus.read(0xFFFE);
                let destination_address_high = bus.read(0xFFFF);
                self.program_counter =
                    (destination_address_high as u16) << 8 | destination_address_low as u16;
                _cycles = 7;
            }
            0x02 => {
                // HTL
                self.halted = true;
            }
            0x05 => {
                // ORA Zero Page
                let address = self.read_immediate_addressed(bus);
                self.bitwise_or(bus, address as u16);
                _cycles = 3;
            }
            0x06 => {
                // ASL Zero Page
                let address = self.read_immediate_addressed(bus);
                self.shift_left(bus, address as u16);
                _cycles = 5;
            }
            0x08 => {
                // PHP
                let status = self.get_status_register(true);
                self.push(bus, status);
                _cycles = 3;
            }
            0x09 => {
                // ORA Immediate
                let value = self.read_immediate_addressed(bus);
                self.a |= value;
                self.flag_zero = self.a == 0;
                self.flag_negative = self.a & 0x80 != 0;
                _cycles = 2;
            }
            0x0A => {
                // ASL A
                self.flag_carry = self.a & 0x80 != 0;
                self.a <<= 1;
                self.flag_zero = self.a == 0;
                self.flag_negative = self.a & 0x80 != 0;
                _cycles = 2;
            }
            0x0D => {
                // ORA Absolute
                let address = self.read_absolute_addressed(bus);
                self.bitwise_or(bus, address);
                _cycles = 4;
            }
            0x0E => {
                // ASL Absolute
                let address = self.read_absolute_addressed(bus);
                self.shift_left(bus, address);
                _cycles = 6;
            }
            0x10 => {
                // BPL
                let offset = self.read_immediate_addressed(bus);
                _cycles = self.branch(!self.flag_negative, offset);
            }
            0x18 => {
                // CLC
                self.flag_carry = false;
                _cycles = 2;
            }
            0x20 => {
                // JSR
                let destination_address_low = bus.read(self.program_counter);
                self.program_counter = self.program_counter.wrapping_add(1);
                let destination_address_high = bus.read(self.program_counter);
                self.push(bus, (self.program_counter >> 8) as u8);
                self.push(bus, self.program_counter as u8);
                self.program_counter =
                    (destination_address_high as u16) << 8 | destination_address_low as u16;
                _cycles = 6;
            }
            0x24 => {
                // BIT Zero Page
                let address = self.read_immediate_addressed(bus);
                let value = bus.read(address as u16);
                self.bit_test(value);
                _cycles = 3;
            }
            0x25 => {
                // AND Zero Page
                let address = self.read_immediate_addressed(bus);
                self.bitwise_and(bus, address as u16);
                _cycles = 3;
            }
            0x26 => {
                // ROL Zero Page
                let address = self.read_immediate_addressed(bus);
                self.rotate_left(bus, address as u16);
                _cycles = 5;
            }
            0x28 => {
                // PLP
                let status = self.pull(bus);
                self.set_status_register(status);
                _cycles = 4;
            }
            0x29 => {
                // AND Immediate
                let value = self.read_immediate_addressed(bus);
                self.a &= value;
                self.flag_zero = self.a == 0;
                self.flag_negative = self.a & 0x80 != 0;
                _cycles = 2;
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
                _cycles = 2;
            }
            0x2C => {
                // BIT Absolute
                let address = self.read_absolute_addressed(bus);
                let value = bus.read(address);
                self.bit_test(value);
                _cycles = 4;
            }
            0x2D => {
                // AND Absolute
                let address = self.read_absolute_addressed(bus);
                self.bitwise_and(bus, address);
                _cycles = 4;
            }
            0x2E => {
                // ROL Absolute
                let address = self.read_absolute_addressed(bus);
                self.rotate_left(bus, address);
                _cycles = 6;
            }
            0x30 => {
                // BMI
                let offset = self.read_immediate_addressed(bus);
                _cycles = self.branch(self.flag_negative, offset);
            }
            0x38 => {
                // SEC
                self.flag_carry = true;
                _cycles = 2;
            }
            0x40 => {
                // RTI
                let status = self.pull(bus);
                self.set_status_register(status);

                let return_address_low = self.pull(bus);
                let return_address_high = self.pull(bus);
                self.program_counter =
                    (return_address_high as u16) << 8 | return_address_low as u16;
                _cycles = 6;
            }
            0x45 => {
                // EOR Zero Page
                let address = self.read_immediate_addressed(bus);
                self.bitwise_eor(bus, address as u16);
                _cycles = 3;
            }
            0x46 => {
                // LSR Zero Page
                let address = self.read_immediate_addressed(bus);
                self.shift_right(bus, address as u16);
                _cycles = 5;
            }
            0x48 => {
                // PHA
                self.push(bus, self.a);
                _cycles = 3;
            }
            0x49 => {
                // EOR Immediate
                let value = self.read_immediate_addressed(bus);
                self.a ^= value;
                self.flag_zero = self.a == 0;
                self.flag_negative = self.a & 0x80 != 0;
                _cycles = 2;
            }
            0x4A => {
                // LSR A
                self.flag_carry = self.a & 1 != 0;
                self.a >>= 1;
                self.flag_zero = self.a == 0;
                self.flag_negative = self.a & 0x80 != 0;
                _cycles = 2;
            }
            0x4C => {
                // JMP
                self.program_counter = self.read_absolute_addressed(bus);
                _cycles = 3;
            }
            0x4D => {
                // EOR Absolute
                let address = self.read_absolute_addressed(bus);
                self.bitwise_eor(bus, address);
                _cycles = 4;
            }
            0x4E => {
                // LSR Absolute
                let address = self.read_absolute_addressed(bus);
                self.shift_right(bus, address);
                _cycles = 6;
            }
            0x50 => {
                // BVC
                let offset = self.read_immediate_addressed(bus);
                _cycles = self.branch(!self.flag_overflow, offset);
            }
            0x58 => {
                // CLI
                self.flag_interrupt_disable = false;
                _cycles = 2;
            }
            0x60 => {
                // RTS
                let return_address_low = self.pull(bus);
                let return_address_high = self.pull(bus);
                self.program_counter =
                    (return_address_high as u16) << 8 | return_address_low as u16;
                self.program_counter = self.program_counter.wrapping_add(1);
                _cycles = 6;
            }
            0x65 => {
                // ADC Zero Page
                let address = self.read_immediate_addressed(bus);
                let value = bus.read(address as u16);
                self.add_carry(value);
                _cycles = 3;
            }
            0x66 => {
                // ROR Zero Page
                let address = self.read_immediate_addressed(bus);
                self.rotate_right(bus, address as u16);
                _cycles = 5;
            }
            0x68 => {
                // PLA
                self.a = self.pull(bus);
                self.flag_zero = self.a == 0;
                self.flag_negative = self.a & 0x80 != 0;
                _cycles = 4;
            }
            0x69 => {
                // ADC Immediate
                let value = self.read_immediate_addressed(bus);
                self.add_carry(value);
                _cycles = 2;
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
                _cycles = 2;
            }
            0x6C => {
                // JMP Indirect
                let value = self.read_indirect_addressed(bus);
                self.program_counter = value;
                _cycles = 5;
            }
            0x6D => {
                // ADC Absolute
                let address = self.read_absolute_addressed(bus);
                let value = bus.read(address);
                self.add_carry(value);
                _cycles = 4;
            }
            0x6E => {
                // ROR Absolute
                let address = self.read_absolute_addressed(bus);
                self.rotate_right(bus, address);
                _cycles = 6;
            }
            0x70 => {
                // BVS
                let offset = self.read_immediate_addressed(bus);
                _cycles = self.branch(self.flag_overflow, offset);
            }
            0x78 => {
                // SEI
                self.flag_interrupt_disable = true;
                _cycles = 2;
            }
            0x84 => {
                // STY Zero Page
                let destination_address = self.read_immediate_addressed(bus);
                bus.write(destination_address as u16, self.y);
                _cycles = 3;
            }
            0x85 => {
                // STA Zero Page
                let destination_address = self.read_immediate_addressed(bus);
                bus.write(destination_address as u16, self.a);
                _cycles = 3;
            }
            0x86 => {
                // STX Zero Page
                let destination_address = self.read_immediate_addressed(bus);
                bus.write(destination_address as u16, self.x);
                _cycles = 3;
            }
            0x88 => {
                // DEY
                self.y = self.y.wrapping_sub(1);
                self.flag_zero = self.y == 0;
                self.flag_negative = self.y & 0x80 != 0;
                _cycles = 2;
            }
            0x8A => {
                // TXA
                self.a = self.x;
                self.flag_zero = self.a == 0;
                self.flag_negative = self.a & 0x80 != 0;
                _cycles = 2;
            }
            0x8C => {
                // STY Absolute
                let destination_address = self.read_absolute_addressed(bus);
                bus.write(destination_address, self.y);
                _cycles = 4;
            }
            0x8D => {
                // STA Absolute
                let destination_address = self.read_absolute_addressed(bus);
                bus.write(destination_address, self.a);
                _cycles = 4;
            }
            0x8E => {
                // STX Absolute
                let destination_address = self.read_absolute_addressed(bus);
                bus.write(destination_address, self.x);
                _cycles = 4;
            }
            0x90 => {
                // BCC
                let offset = self.read_immediate_addressed(bus);
                _cycles = self.branch(!self.flag_carry, offset);
            }
            0x98 => {
                // TYA
                self.a = self.y;
                self.flag_zero = self.a == 0;
                self.flag_negative = self.a & 0x80 != 0;
                _cycles = 2;
            }
            0x9A => {
                // TXS
                self.stack_pointer = self.x;
                _cycles = 2;
            }
            0xA0 => {
                // LDY Immediate
                self.y = self.read_immediate_addressed(bus);
                self.flag_zero = self.y == 0;
                self.flag_negative = self.y & 0x80 != 0;
                _cycles = 2;
            }
            0xA2 => {
                // LDX Immediate
                self.x = self.read_immediate_addressed(bus);
                self.flag_zero = self.x == 0;
                self.flag_negative = self.x & 0x80 != 0;
                _cycles = 2;
            }
            0xA5 => {
                // LDA Zero Page
                let destination_address = self.read_immediate_addressed(bus);
                self.a = bus.read(destination_address as u16);
                self.flag_zero = self.a == 0;
                self.flag_negative = self.a & 0x80 != 0;
                _cycles = 3;
            }
            0xA8 => {
                // TAY
                self.y = self.a;
                self.flag_zero = self.y == 0;
                self.flag_negative = self.y & 0x80 != 0;
                _cycles = 2;
            }
            0xA9 => {
                // LDA Immediate
                self.a = self.read_immediate_addressed(bus);
                self.flag_zero = self.a == 0;
                self.flag_negative = self.a & 0x80 != 0;
                _cycles = 2;
            }
            0xAA => {
                // TAX
                self.x = self.a;
                self.flag_zero = self.x == 0;
                self.flag_negative = self.x & 0x80 != 0;
                _cycles = 2;
            }
            0xAC => {
                // LDY Absolute
                let destination_address = self.read_absolute_addressed(bus);
                self.y = bus.read(destination_address);
                self.flag_zero = self.y == 0;
                self.flag_negative = self.y & 0x80 != 0;
                _cycles = 4;
            }
            0xAD => {
                // LDA Absolute
                let destination_address = self.read_absolute_addressed(bus);
                self.a = bus.read(destination_address);
                self.flag_zero = self.a == 0;
                self.flag_negative = self.a & 0x80 != 0;
                _cycles = 4;
            }
            0xAE => {
                // LDX Absolute
                let destination_address = self.read_absolute_addressed(bus);
                self.x = bus.read(destination_address);
                self.flag_zero = self.x == 0;
                self.flag_negative = self.x & 0x80 != 0;
                _cycles = 4;
            }
            0xB0 => {
                // BCS
                let offset = self.read_immediate_addressed(bus);
                _cycles = self.branch(self.flag_carry, offset);
            }
            0xB8 => {
                // CLV
                self.flag_overflow = false;
                _cycles = 2;
            }
            0xBA => {
                // TSX
                self.x = self.stack_pointer;
                self.flag_zero = self.x == 0;
                self.flag_negative = self.x & 0x80 != 0;
                _cycles = 2;
            }
            0xC0 => {
                // CPY Immediate
                let value = self.read_immediate_addressed(bus);
                self.compare_y(value);
                _cycles = 2;
            }
            0xC4 => {
                // CPY Zero Page
                let address = self.read_immediate_addressed(bus);
                let value = bus.read(address as u16);
                self.compare_y(value);
                _cycles = 3;
            }
            0xC5 => {
                // CMP Zero Page
                let address = self.read_immediate_addressed(bus);
                let value = bus.read(address as u16);
                self.compare_a(value);
                _cycles = 3;
            }
            0xC6 => {
                // DEC Zero Page
                let address = self.read_immediate_addressed(bus);
                self.decrement(bus, address as u16);
                _cycles = 5;
            }
            0xC8 => {
                // INY
                self.y = self.y.wrapping_add(1);
                self.flag_zero = self.y == 0;
                self.flag_negative = self.y & 0x80 != 0;
                _cycles = 2;
            }
            0xC9 => {
                // CMP Immediate
                let value = self.read_immediate_addressed(bus);
                self.compare_a(value);
                _cycles = 2;
            }
            0xCA => {
                // DEX
                self.x = self.x.wrapping_sub(1);
                self.flag_zero = self.x == 0;
                self.flag_negative = self.x & 0x80 != 0;
                _cycles = 2;
            }
            0xCC => {
                // CPY Absolute
                let address = self.read_absolute_addressed(bus);
                let value = bus.read(address);
                self.compare_y(value);
                _cycles = 4;
            }
            0xCD => {
                // CMP Absolute
                let address = self.read_absolute_addressed(bus);
                let value = bus.read(address);
                self.compare_a(value);
                _cycles = 4;
            }
            0xCE => {
                // DEC Absolute
                let address = self.read_absolute_addressed(bus);
                self.decrement(bus, address);
                _cycles = 6;
            }
            0xD0 => {
                // BNE
                let offset = self.read_immediate_addressed(bus);
                _cycles = self.branch(!self.flag_zero, offset);
            }
            0xD8 => {
                // CLD
                self.flag_decimal = false;
                _cycles = 2;
            }
            0xE0 => {
                // CPX Immediate
                let value = self.read_immediate_addressed(bus);
                self.compare_x(value);
                _cycles = 2;
            }
            0xE4 => {
                // CPX Zero Page
                let address = self.read_immediate_addressed(bus);
                let value = bus.read(address as u16);
                self.compare_x(value);
                _cycles = 3;
            }
            0xE5 => {
                // SBC Zero Page
                let address = self.read_immediate_addressed(bus);
                let value = bus.read(address as u16);
                self.sub_carry(value);
                _cycles = 3;
            }
            0xE6 => {
                // INC Zero Page
                let address = self.read_immediate_addressed(bus);
                self.increment(bus, address as u16);
                _cycles = 5;
            }
            0xE8 => {
                // INX
                self.x = self.x.wrapping_add(1);
                self.flag_zero = self.x == 0;
                self.flag_negative = self.x & 0x80 != 0;
                _cycles = 2;
            }
            0xE9 => {
                // SBC Immediate
                let value = self.read_immediate_addressed(bus);
                self.sub_carry(value);
                _cycles = 2;
            }
            0xEA => {
                // NOP
                _cycles = 2;
            }
            0xEC => {
                // CPX Absolute
                let address = self.read_absolute_addressed(bus);
                let value = bus.read(address);
                self.compare_x(value);
                _cycles = 4;
            }
            0xED => {
                // SBC Absolute
                let address = self.read_absolute_addressed(bus);
                let value = bus.read(address);
                self.sub_carry(value);
                _cycles = 4;
            }
            0xEE => {
                // INC Absolute
                let address = self.read_absolute_addressed(bus);
                self.increment(bus, address);
                _cycles = 6;
            }
            0xF0 => {
                // BEQ
                let offset = self.read_immediate_addressed(bus);
                _cycles = self.branch(self.flag_zero, offset);
            }
            0xF8 => {
                // SED
                self.flag_decimal = true;
                _cycles = 2;
            }
            _ => {
                // Unknown opcode
                panic!("Unknown opcode {:02X}", opcode)
            }
        }
    }
}
