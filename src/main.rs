use image::ColorType::Rgb8;
use nintendrust::bus::Bus;
use nintendrust::cpu::Cpu;
use nintendrust::rom::Rom;
use std::fs;

fn main() {
    let file_path = "Super Mario Bros.nes";
    let raw_bytes = match fs::read(file_path) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Could not read file: {}", e);
            return;
        }
    };

    let rom = Rom::new(&raw_bytes);

    let mut bus = Bus::new(rom);
    let mut cpu = Cpu::new();
    let frame = bus.ppu.debug_draw_pattern_tables();
    image::save_buffer("pattern_tables.png", &frame, 256, 128, Rgb8).expect("Failed to save image");

    cpu.reset(&mut bus);

    // while !cpu.halted {
    //     println!("{}", cpu.trace(&bus));
    //     cpu.emulate_cpu(&mut bus);
    // }
}
