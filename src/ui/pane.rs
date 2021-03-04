use tui::widgets::{Paragraph, Block, Borders};
use tui::text::Text;

use crate::non_blocking::Command;

#[derive(Debug)]
pub struct Pane {
    output: Vec<u8>,
    offset: usize,
    receiver: Receiver<Vec<u8>>,
    sender: Sender<Vec<u8>>
}

impl Pane {
    pub fn new(command: Command) -> Self{
        Self {
            output: vec![],
            offset: 0,
            receiver: receiver,
            sender: sender
        }
    }

    pub fn get_output_as_paragraph(&self) -> Paragraph {
        let text = ansi4tui::bytes_to_text(&self.output);
        let paragraph = Paragraph::new(text).block(Block::default().borders(Borders::ALL)).scroll((self.offset as u16, 0));
            command
        }
    }

    pub fn get_output_as_paragraph(&mut self) -> Paragraph {
        let text = match self.command.get_output(){
            Ok(output) => ansi4tui::bytes_to_text(&output),
            Err(err) => Text::from(err)
        };

        let paragraph = Paragraph::new(text).block(Block::default().borders(Borders::ALL));

        paragraph
    }

    pub fn scroll_down(&mut self) {
        self.offset += 1;
    }
    pub fn scroll_up(&mut self) {
        if self.offset > 1 {
            self.offset -= 1;
        }
    }

    
    pub fn send(&mut self, message: char) -> Result<(), String> {
        match self.command.send_char(message){
            Ok(_) => Ok(()),
            Err(err) => Err(err.to_string())
        }
    }
    
    /// Kills the underlying process
    pub fn kill_process(&mut self) {
        self.command.kill_process()
    }
}
