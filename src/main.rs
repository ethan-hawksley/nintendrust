use nintendrust::bus::Bus;
use nintendrust::cpu::Cpu;
use nintendrust::rom::Rom;
use std::fs;

fn main() {
    let file_path = "5_Instructions1.nes";
    let raw_bytes = match fs::read(file_path) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Could not read file: {}", e);
            return;
        }
    };

    let rom = match Rom::new(&raw_bytes) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("ROM Error: {}", e);
            return;
        }
    };

    let mut bus = Bus::new();
    let mut cpu = Cpu::new();

    bus.insert_cartridge(rom);

    cpu.reset(&mut bus);

    while !cpu.halted {
        println!("{}", cpu.trace(&bus));
        cpu.emulate_cpu(&mut bus);
    }
}
