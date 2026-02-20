use image::ColorType::Rgb8;
use minifb::{Key, Scale, Window, WindowOptions};
use nintendrust::bus::Bus;
use nintendrust::cpu::Cpu;
use nintendrust::rom::Rom;
use std::fs;

fn main() {
    let file_path = "Zooming_Secretary.nes";
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

    let mut window = Window::new(
        "Nintendrust",
        256,
        240,
        WindowOptions {
            scale: Scale::X2,
            ..WindowOptions::default()
        },
    )
    .unwrap();

    window.set_target_fps(60);

    while window.is_open() {
        bus.ppu.frame_complete = false;
        bus.controller.reset_controllers();
        if window.is_key_down(Key::Right) {
            bus.controller.right_p1();
        }
        if window.is_key_down(Key::Left) {
            bus.controller.left_p1();
        }
        if window.is_key_down(Key::Down) {
            bus.controller.down_p1();
        }
        if window.is_key_down(Key::Up) {
            bus.controller.up_p1();
        }
        if window.is_key_down(Key::Z) {
            bus.controller.a_p1();
        }
        if window.is_key_down(Key::X) {
            bus.controller.b_p1();
        }
        if window.is_key_down(Key::Enter) {
            bus.controller.start_p1();
        }
        if window.is_key_down(Key::Space) {
            bus.controller.select_p1();
        }
        while !bus.ppu.frame_complete && !cpu.halted {
            cpu.emulate_cpu(&mut bus);
        }
        window
            .update_with_buffer(&bus.ppu.frame_buffer, 256, 240)
            .unwrap();
    }

    // for _ in 0..1000000 {
    //     // println!("{}", cpu.trace(&bus));
    //     cpu.emulate_cpu(&mut bus);
    // }

    // while !cpu.halted {
    //     cpu.emulate_cpu(&mut bus);
    // }

    // let output_frame = bus.ppu.debug_draw_nametable();
    // image::save_buffer("nametable.png", &output_frame, 512, 240, Rgb8)
    //     .expect("Failed to save image");
}
