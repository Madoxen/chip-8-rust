pub trait Chip8Display {
    fn display(&self, data: [[bool; 64]; 32]);
}

pub struct Chip8TerminalDisplay {}
pub struct Chip8NullDisplay {}

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
    fn display(&self, data: [[bool; 64]; 32]) {
        print!("{esc}c", esc = 27 as char);
        print!("\r");
        data.iter().for_each(|row| {
            println!();
            row.iter().for_each(|pix| {
                if *pix == true {
                    print!("\u{25A0}");
                } else {
                    print!(" ");
                }
            });
        });
    }
}


impl Chip8Display for Chip8NullDisplay {
    fn display(&self, data: [[bool; 64]; 32]) {
        
    }
}
