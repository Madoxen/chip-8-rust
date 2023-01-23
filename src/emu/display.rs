pub trait Chip8Display {
    fn display(&self, data: [[bool; 32]; 64]);
}

pub struct Chip8TerminalDisplay {}

impl Chip8TerminalDisplay {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Chip8TerminalDisplay {
    fn default() -> Self {
        Self::new()
    }
}

impl Chip8Display for Chip8TerminalDisplay {
    fn display(&self, data: [[bool; 32]; 64]) {
        println!();
        data.iter().for_each(|row| {
            println!();
            row.iter().for_each(|pix| {
                if *pix == true {
                    print!("#");
                } else {
                    print!(" ");
                }
            });
        });
        println!();
    }
}
