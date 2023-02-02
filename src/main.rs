use std::fs::File;
use std::io::Read;
use std::{env, vec};
mod emu;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Provide path to chip-8 ROM");
        std::process::exit(1);
    }

    let rom_path = &args[1];
    let f = match File::open(rom_path) {
        Ok(v) => v,
        Err(v) => panic!("Could not open file: {}", v),
    };

    let program_bytes: Vec<u8> = f
        .bytes()
        .map(|x| match x {
            Ok(v) => v,
            Err(_) => panic!("Error on program read"),
        })
        .collect();

    let mut emu = emu::Chip8Emulator::new(&program_bytes, emu::display::Chip8TerminalDisplay {});
    emu.run(700);
}
