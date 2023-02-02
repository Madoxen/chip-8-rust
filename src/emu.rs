pub mod display;
use core::time;
use std::io::Error;
use getch::Getch;
use rand::prelude::*;
use std::num::Wrapping;
use std::sync::atomic::AtomicU8;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, sleep, sleep_ms, JoinHandle};

pub struct Chip8Emulator<D: display::Chip8Display> {
    memory: [u8; 4096],
    pc: usize,
    curr_instr: [u8; 2],
    registers: [u8; 16],
    stack: Vec<usize>,
    index: usize,
    video_mem: [[bool; 64]; 32],
    running: bool,
    display_dev: D,
    delay_timer: Arc<Mutex<u8>>,
    sound_timer: Arc<Mutex<u8>>,
    keyboard_channel: Option<Receiver<Result<u8, Error>>>,
}

const PROGRAM_MEMORY_OFFSET: usize = 0x200;
const FONT_MEMORY_OFFSET: usize = 0x50;

impl<D: display::Chip8Display> Chip8Emulator<D> {
    pub fn new(program: &Vec<u8>, display_dev: D) -> Self {
        let mut memory = [0; 4096];
        for i in 0..program.len() {
            memory[i + PROGRAM_MEMORY_OFFSET] = program[i];
        }

        let font: [u8; 80] = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80,
        ]; // F

        for i in 0..font.len() {
            memory[i + FONT_MEMORY_OFFSET] = font[i];
        }

        Self {
            running: false,
            pc: PROGRAM_MEMORY_OFFSET,
            curr_instr: [0, 0],
            memory,
            registers: [0; 16],
            index: 0,
            video_mem: [[false; 64]; 32],
            display_dev,
            stack: vec![],
            delay_timer: Arc::new(Mutex::new(0)),
            sound_timer: Arc::new(Mutex::new(0)),
            keyboard_channel: None
        }
    }

    pub fn run(&mut self) {
        //start timer thread
        self.running = true;
        let delay_timer = Arc::clone(&self.delay_timer);
        let sound_timer = Arc::clone(&self.sound_timer);
        let timer_join_handle = thread::spawn(move || {
            loop {
                //Open scope for earlier dropping
                {
                    let mut delay_timer = delay_timer.lock().unwrap();
                    let mut sound_timer = sound_timer.lock().unwrap();
                    *delay_timer += 1;
                    *sound_timer += 1;
                }
                sleep(time::Duration::from_millis(16));
            }
        });

        let (tx, rx) = channel();
        self.keyboard_channel = Some(rx);

        let keyboard_join_handle = thread::spawn(move || loop {
            let g = Getch::new();
            match tx.send(g.getch()) {
                Ok(_) => (),
                Err(_) => println!("Keyboard Thread: TX failed, no data will be sent"),
            };
        });

        while self.running {
            self.fetch();
            self.decode();
        }

        keyboard_join_handle.join();
        timer_join_handle.join();
    }

    fn fetch(&mut self) {
        self.curr_instr[0] = self.memory[self.pc];
        self.pc += 1;
        self.curr_instr[1] = self.memory[self.pc];
        self.pc += 1;
    }

    fn decode(&mut self) {
        let op: u8 = (self.curr_instr[0] & 0b11110000) >> 4;
        let x: usize = (self.curr_instr[0] & 0b00001111) as usize;
        let y: usize = ((self.curr_instr[1] & 0b11110000) >> 4) as usize;
        let n: u8 = self.curr_instr[1] & 0b00001111;

        let nn: u8 = self.curr_instr[1];
        //produce nnn => 0bxxxxNNNNNNNN
        let nnn: u16 = (x as u16) << 8 | (nn as u16);

        let val_x: u8 = self.registers[x];
        let val_y: u8 = self.registers[y];

        match op {
            0x0 => match x {
                0x0 => match y {
                    0xE => match n {
                        0x0 => self.op_00e0_cls(),
                        0xE => self.op_00ee_ret(),
                        _ => panic!("No opcode 0x{:x}{:x}{:x}{:x}", op, x, y, n),
                    },
                    _ => panic!("No opcode 0x{:x}{:x}{:x}{:x}", op, x, y, n),
                },
                _ => panic!("No opcode 0x{:x}{:x}{:x}{:x}", op, x, y, n),
            },
            0x1 => self.op_1nnn_jump(nnn),
            0x2 => self.op_2nnn_call(nnn),
            0x3 => self.op_3xnn_skip(val_x, nn),
            0x4 => self.op_4xnn_skip(val_x, nn),
            0x5 => match n {
                0x0 => self.op_5xy0_skip(val_x, val_y),
                _ => panic!("No opcode 0x{:x}{:x}{:x}{:x}", op, x, y, n),
            },
            0x6 => self.registers[x as usize] = nn,
            0x7 => self.op_7xnn_add(x, nn),
            0x8 => match n {
                0x0 => self.op_8xy0_set(x, y),
                0x1 => self.op_8xy1_or(x, y),
                0x2 => self.op_8xy2_and(x, y),
                0x3 => self.op_8xy3_xor(x, y),
                0x4 => self.op_8xy4_add(x, y),
                0x5 => self.op_8xy5_sub(x, y),
                0x6 => self.op_8xy6_rshift(x, y),
                0x7 => self.op_8xy7_sub_rev(x, y),
                0xE => self.op_8xye_lshift(x, y),

                _ => panic!("No opcode 0x{:x}{:x}{:x}{:x}", op, x, y, n),
            },
            0x9 => match n {
                0x0 => self.op_9xy0_skip(val_x, val_y),
                _ => panic!("No opcode 0x{:x}{:x}{:x}{:x}", op, x, y, n),
            },
            0xA => self.index = nnn as usize,
            0xB => self.op_bnnn_jumpoff(nnn),
            0xC => self.op_cxnn_rand(x, nn),
            0xD => self.op_dxyn_disp(x, y, n),
            0xF => match nn {
                0x15 => self.op_fx15_set_delay(val_x),
                0x18 => self.op_fx18_set_sound(val_x),
                0x07 => self.op_fx07_xtdtime(x),
                0x33 => self.op_fx33_conv(val_x),
                0x55 => self.op_fx55_store(x),
                0x65 => self.op_fx65_load(x),
                0x1e => self.op_fx1e_add_idx(val_x),
                _ => panic!("No opcode 0x{:x}{:x}{:x}{:x}", op, x, y, n),
            },
            _ => panic!("No opcode 0x{:x}{:x}{:x}{:x}", op, x, y, n),
        }
    }

    fn op_00e0_cls(&mut self) {
        for y in 0..32 {
            for x in 0..64 {
                self.video_mem[y][x] = false;
            }
        }

        self.display_dev.display(self.video_mem);
    }

    fn op_00ee_ret(&mut self) {
        let npc = match self.stack.pop() {
            Some(v) => v,
            None => panic!("FATAL: tried to return while stack being empty"),
        };
        self.pc = npc;
    }

    fn op_1nnn_jump(&mut self, nnn: u16) {
        self.pc = nnn as usize
    }

    fn op_2nnn_call(&mut self, nnn: u16) {
        self.stack.push(self.pc);
        self.pc = nnn as usize
    }

    fn op_3xnn_skip(&mut self, val_x: u8, nn: u8) {
        if val_x == nn {
            self.pc += 2;
        }
    }

    fn op_4xnn_skip(&mut self, val_x: u8, nn: u8) {
        if val_x != nn {
            self.pc += 2;
        }
    }

    fn op_5xy0_skip(&mut self, val_x: u8, val_y: u8) {
        if val_x == val_y {
            self.pc += 2;
        }
    }

    fn op_9xy0_skip(&mut self, val_x: u8, val_y: u8) {
        if val_x != val_y {
            self.pc += 2;
        }
    }

    fn op_7xnn_add(&mut self, x: usize, nn: u8) {
        let x_val = Wrapping(self.registers[x]);
        let nn = Wrapping(nn);

        self.registers[x] = (x_val + nn).0;
    }

    fn op_8xy0_set(&mut self, x: usize, y: usize) {
        self.registers[x] = self.registers[y];
    }

    fn op_8xy1_or(&mut self, x: usize, y: usize) {
        self.registers[x] |= self.registers[y];
    }

    fn op_8xy2_and(&mut self, x: usize, y: usize) {
        self.registers[x] &= self.registers[y];
    }

    fn op_8xy3_xor(&mut self, x: usize, y: usize) {
        self.registers[x] ^= self.registers[y];
    }

    fn op_8xy4_add(&mut self, x: usize, y: usize) {
        let x_val = self.registers[x];
        let y_val = self.registers[y];

        //Detect overflow
        //check if x + y > 255
        self.registers[0xF] = 0;
        if y_val > u8::MAX - x_val || x_val > u8::MAX - y_val {
            self.registers[0xF] = 1
        }

        self.registers[x] = (Wrapping(self.registers[x]) + Wrapping(self.registers[y])).0;
    }

    fn op_8xy5_sub(&mut self, x: usize, y: usize) {
        let x_val = self.registers[x];
        let y_val = self.registers[y];

        //Detect overflow
        //check if x - y < 0
        // x < y
        self.registers[0xF] = 1;
        if x_val < y_val {
            self.registers[0xF] = 0;
        }

        self.registers[x] = (Wrapping(self.registers[x]) + Wrapping(self.registers[y])).0;
    }

    fn op_8xy6_rshift(&mut self, x: usize, y: usize) {
        //Set F register to the shifted out bit value
        self.registers[0xF] = self.registers[x] & 1;
        self.registers[x] >>= 1;
    }

    fn op_8xy7_sub_rev(&mut self, x: usize, y: usize) {
        let x_val = self.registers[x];
        let y_val = self.registers[y];

        //Detect overflow
        //check if y < x
        self.registers[0xF] = 1;
        if y_val < x_val {
            self.registers[0xF] = 0;
        }

        self.registers[y] -= self.registers[x];
    }

    fn op_8xye_lshift(&mut self, x: usize, y: usize) {
        //Set F register to the shifted out bit value
        self.registers[0xF] = (self.registers[x] & 0b10000000) >> 7;
        self.registers[x] <<= 1;
    }

    fn op_bnnn_jumpoff(&mut self, nnn: u16) {
        self.pc = nnn as usize + self.registers[0] as usize;
    }

    fn op_cxnn_rand(&mut self, x: usize, nn: u8) {
        let mut r = rand::random::<u8>();
        r &= nn;
        self.registers[x] = r;
    }

    fn op_dxyn_disp(&mut self, vx: usize, vy: usize, n_val: u8) {
        let x_coord = self.registers[vx] % 64;
        let y_coord: u8 = self.registers[vy] % 32;
        //Set collision flag to 0
        self.registers[0xF] = 0;

        for row_idx in 0..n_val {
            let mem_idx = self.index + row_idx as usize;
            let sprite_row = self.memory[mem_idx];
            for pix_idx in 0..8 {
                let pix = sprite_row & (0b10000000 >> pix_idx);
                let x = (x_coord + pix_idx) as usize;
                let y = (y_coord + row_idx) as usize;

                if x >= 64 || y >= 32 {
                    break;
                }

                if pix != 0 {
                    if self.video_mem[y][x] {
                        self.registers[0xF] = 1;
                    }
                }

                self.video_mem[y][x] ^= pix != 0;
            }
        }

        self.display_dev.display(self.video_mem);
    }

    fn op_fx07_xtdtime(&mut self, x: usize) {
        let delay_timer_val = *self.delay_timer.lock().unwrap();
        self.registers[x] = delay_timer_val;
    }

    fn op_fx0a_get_key(&mut self, x: usize) {
        let rx = match &self.keyboard_channel {
            Some(v) => v,
            None => panic!("Keyboard channel does not exist, program cannot continue, aborting."),
        };

        self.registers[x] = match rx.recv() {
            Ok(v) => match v {
                Ok(keycode) => keycode,
                Err(_) => todo!(),
            },
            Err(_) => todo!(),
        };
    }

    fn op_fx15_set_delay(&mut self, val_x: u8) {
        let mut delay_timer = self.delay_timer.lock().unwrap();
        *delay_timer = val_x;
    }

    fn op_fx18_set_sound(&mut self, val_x: u8) {
        let mut sound_timer = self.sound_timer.lock().unwrap();
        *sound_timer = val_x;
    }

    fn op_fx1e_add_idx(&mut self, val_x: u8) {
        self.index += val_x as usize;
        self.registers[0xF] = if self.index > 0x0F00 { 1 } else { 0 };
    }
    /// Store BCD representation of Vx in memory locations I, I+1, and I+2.
    /// The interpreter takes the decimal value of Vx,
    /// and places the hundreds digit in memory at location in I,
    /// the tens digit at location I+1, and the ones digit at location I+2.
    fn op_fx33_conv(&mut self, val_x: u8) {
        let i = self.index;
        self.memory[i] = val_x / 100 % 10;
        self.memory[i + 1] = val_x / 10 % 10;
        self.memory[i + 2] = val_x % 10;
    }

    /// FX55, the value of each variable register from V0 to VX inclusive
    /// (if X is 0, then only V0) will be stored in successive memory addresses,
    /// starting with the one that’s stored in I.
    /// V0 will be stored at the address in I,
    /// V1 will be stored in I + 1, and so on, until VX is stored in I + X.
    fn op_fx55_store(&mut self, x: usize) {
        for i in 0..(x + 1) {
            let idx = self.index + i;
            self.memory[idx] = self.registers[i];
        }
    }

    /// FX65, loads data to the V0-VX (inclusive) registers from memory starting at
    /// address stored at I
    fn op_fx65_load(&mut self, x: usize) {
        for i in 0..(x + 1) {
            let idx = self.index + i;
            self.registers[i] = self.memory[idx];
        }
    }
}
