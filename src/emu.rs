pub mod display;
use std::num::Wrapping;
pub struct Chip8Emulator<D: display::Chip8Display> {
    memory: [u8; 4096],
    pc: usize,
    curr_instr: [u8; 2],
    registers: [u8; 16],
    stack: Vec<usize>,
    index: u16,
    video_mem: [[bool; 64]; 32],
    running: bool,
    display_dev: D,
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
        }
    }

    pub fn run(&mut self) {
        self.running = true;
        while self.running {
            self.fetch();
            self.decode();
        }
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
        let nnn: u16 = 0x0 | (x as u16) << 8 | (nn as u16);

        let val_x: u8 = self.registers[x];
        let val_y: u8 = self.registers[y];

        match op {
            0x0 => match x {
                0x0 => match y {
                    0xE => match n {
                        0x0 => self._00E0_cls(),
                        0xE => self._00EE_ret(),
                        _ => panic!("No opcode 0x{:x}{:x}{:x}{:x}", op, x, y, n)
                    },
                    _ => panic!("No opcode 0x{:x}{:x}{:x}{:x}", op, x, y, n)
                },
                _ => panic!("No opcode 0x{:x}{:x}{:x}{:x}", op, x, y, n)
            },
            0x1 => self._1NNN_jump(nnn),
            0x2 => self._2NNN_call(nnn),
            0x3 => self._3XNN_skip(val_x, nn),
            0x4 => self._4XNN_skip(val_x, nn),
            0x5 => match n {
                0x0 => self._5XY0_skip(val_x, val_y),
                _ => panic!("No opcode 0x{:x}{:x}{:x}{:x}", op, x, y, n)
            },
            0x6 => self.registers[x as usize] = nn,
            0x7 => self._7XNN_add(x, nn),
            0x8 => match n {
                0x0 => self._8XY0_set(x, y),
                0x1 => self._8XY1_or(x, y),
                0x2 => self._8XY2_and(x, y),
                0x3 => self._8XY3_xor(x, y),
                0x4 => self._8XY4_add(x, y),
                0x5 => self._8XY5_sub(x, y),
                0x6 => self._8XY6_rshift(x, y),
                0x7 => self._8XY7_sub_rev(x, y),
                0xE => self._8XYE_lshift(x, y),
                _ => panic!("No opcode 0x{:x}{:x}{:x}{:x}", op, x, y, n)
            },
            0x9 => match n {
                0x0 => self._9XY0_skip(val_x, val_y),
                _ => panic!("No opcode 0x{:x}{:x}{:x}{:x}", op, x, y, n)
            },
            0xA => self.index = nnn,
            0xD => self._DXYN_disp(x, y, n),
            _ => panic!("No opcode 0x{:x}{:x}{:x}{:x}", op, x, y, n)
        }
    }

    fn _00E0_cls(&mut self) {
        for y in 0..32 {
            for x in 0..64 {
                self.video_mem[y][x] = false;
            }
        }

        self.display_dev.display(self.video_mem);
    }

    fn _00EE_ret(&mut self) {
        let npc = match self.stack.pop() {
            Some(v) => v,
            None => panic!("FATAL: tried to return while stack being empty"),
        };
        self.pc = npc;
    }

    fn _1NNN_jump(&mut self, nnn: u16) {
        self.pc = nnn as usize
    }

    fn _2NNN_call(&mut self, nnn: u16) {
        self.stack.push(self.pc);
        self.pc = nnn as usize
    }

    fn _3XNN_skip(&mut self, val_x: u8, nn: u8) {
        if val_x == nn {
            self.pc += 2;
        }
    }

    fn _4XNN_skip(&mut self, val_x: u8, nn: u8) {
        if val_x != nn {
            self.pc += 2;
        }
    }

    fn _5XY0_skip(&mut self, val_x: u8, val_y: u8) {
        if val_x == val_y {
            self.pc += 2;
        }
    }

    fn _9XY0_skip(&mut self, val_x: u8, val_y: u8) {
        if val_x != val_y {
            self.pc += 2;
        }
    }

    fn _7XNN_add(&mut self, x: usize, nn: u8) {
        let x_val = Wrapping(self.registers[x]);
        let nn = Wrapping(nn);

        self.registers[x] = (x_val + nn).0;
    }

    fn _8XY0_set(&mut self, x: usize, y: usize) {
        self.registers[x] = self.registers[y];
    }

    fn _8XY1_or(&mut self, x: usize, y: usize) {
        self.registers[x] |= self.registers[y];
    }

    fn _8XY2_and(&mut self, x: usize, y: usize) {
        self.registers[x] &= self.registers[y];
    }

    fn _8XY3_xor(&mut self, x: usize, y: usize) {
        self.registers[x] ^= self.registers[y];
    }

    fn _8XY4_add(&mut self, x: usize, y: usize) {
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

    fn _8XY5_sub(&mut self, x: usize, y: usize) {
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

    fn _8XY6_rshift(&mut self, x: usize, y: usize) {
        //Set F register to the shifted out bit value
        self.registers[0xF] = self.registers[x] & 1;
        self.registers[x] >>= 1;
    }

    fn _8XY7_sub_rev(&mut self, x: usize, y: usize) {
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

    fn _8XYE_lshift(&mut self, x: usize, y: usize) {
        //Set F register to the shifted out bit value
        self.registers[0xF] = (self.registers[x] & 0b10000000) >> 7;
        self.registers[x] <<= 1;
    }

    fn _BNNN_jumpoff(&mut self, nnn: u16) {
        self.pc = nnn as usize + self.registers[0] as usize;
    }


    fn _DXYN_disp(&mut self, vx: usize, vy: usize, n_val: u8) {
        let x_coord = self.registers[vx] % 64;
        let y_coord: u8 = self.registers[vy] % 32;
        //Set collision flag to 0
        self.registers[0xF] = 0;

        for row_idx in 0..n_val {
            let mem_idx = self.index as usize + row_idx as usize;
            let sprite_row = self.memory[mem_idx];
            for pix_idx in 0..8 {
                let pix = sprite_row & (0b10000000 >> pix_idx);
                let x = (x_coord + pix_idx) as usize;
                let y = (y_coord + row_idx) as usize;

                if x >= 64 || y >= 32 {
                    break;
                }

                if pix != 0 {
                    if self.video_mem[y][x] != false {
                        self.registers[0xF] = 1;
                    }
                }

                self.video_mem[y][x] ^= pix != 0;
            }
        }

        self.display_dev.display(self.video_mem);
    }
}
