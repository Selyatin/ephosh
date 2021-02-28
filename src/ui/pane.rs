use tui::widgets::{Paragraph, Block, Borders};
use std::sync::mpsc::{Sender, Receiver};

#[derive(Debug)]
pub struct Pane {
    output: Vec<u8>,
    offset: usize,
    receiver: Receiver<Vec<u8>>,
    sender: Sender<Vec<u8>>
}

impl Pane {
    pub fn new(sender: Sender<Vec<u8>>, receiver: Receiver<Vec<u8>>) -> Self{
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

    
    pub fn send<S: AsRef<str>>(&mut self, message: S) -> Result<(), String> {
        match self.sender.send(message.as_ref().as_bytes().to_vec()){
            Ok(_) => Ok(()),
            Err(err) => Err(err.to_string())
        }
    }
    
    pub fn send_line<S: AsRef<str>>(&mut self, message: S) -> Result<(), String> {
        self.send(message.as_ref())
    }
    
    pub fn recv(&mut self){
        if let Ok(mut output) = self.receiver.try_recv(){
            self.output.append(&mut output);
        }
    }

    /// Kills the underlying process
    pub fn kill_process(&mut self) -> Result<(), String>{
        self.send("01101011 01101001 01101100 01101100")
    }
}
