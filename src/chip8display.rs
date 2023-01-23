pub(crate) trait Chip8Display 
{
    fn display(data: [[bool; 64]; 32]); 
}


pub(crate) struct Chip8TerminalDisplay {}

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
   fn display(data: [[bool; 64]; 32]) {

   }
}