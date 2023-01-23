pub(crate) struct Chip8Emulator {
    memory: [u8; 4096],
    pc: usize,
    curr_instr: [u8; 2],
    registers: [u8; 16],
    index: u16,
    running: bool,
}

const PROGRAM_MEMORY_OFFSET: usize = 0x200;
const FONT_MEMORY_OFFSET: usize = 0x50;

impl Chip8Emulator {
    pub fn new(program: &Vec<u8>, display_dev: Chip8Display) -> Self {

        let mut memory = [0; 4096];
        for i in 0..program.len(){
            memory[i + PROGRAM_MEMORY_OFFSET] = program[i];
        }


        let font : [u8; 80] = [0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
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
        0xF0, 0x80, 0xF0, 0x80, 0x80];  // F


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
        let x: u8 = self.curr_instr[0] & 0b00001111;
        let y: u8 = (self.curr_instr[1] & 0b11110000) >> 4;
        let n: u8 = self.curr_instr[0] & 0b00001111;

        let nn: u8 = self.curr_instr[1];
        //produce nnn => 0bxxxxNNNNNNNN
        let nnn: u16 = 0x0 | (x as u16) << 8 | (nn as u16);


        match op {
            0x0 => match y {
                0xE => self.clear_screen(),
                _ => todo!(),
            },
            0x1 => self.pc = nnn as usize,
            0x6 => self.registers[x as usize] = nn,
            0x7 => self.registers[x as usize] += nn,
            0xA => self.index = nnn,
            0xD => self.display(),
            _ => todo!(),
        }
    }

    fn clear_screen(&mut self) {

    }

    fn display(&mut self) {

    }
}
