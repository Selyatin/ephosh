use tui::widgets::{Paragraph, Block, Borders};
use tui::text::{Spans};
use std::sync::mpsc::{Sender, Receiver};

#[derive(Debug)]
pub struct Pane {
    output: String,
    receiver: Receiver<String>,
    sender: Sender<String>
}

impl Pane {
    pub fn new(sender: Sender<String>, receiver: Receiver<String>) -> Self{
        Self {
            output: String::new(),
            receiver: receiver,
            sender: sender
        }
    }

    pub fn _get_output(&self) -> &str {
        &self.output
    }

    pub fn get_output_as_paragraph(&self) -> Paragraph {
        let new_lines: Vec<&str> = self.output.split("\n").collect();
        
        let mut spans_lines: Vec<Spans> = vec![];

        for line in new_lines {
            let spans = Spans::from(line);
            spans_lines.push(spans);
        }

        let paragraph = Paragraph::new(spans_lines).block(Block::default().borders(Borders::ALL));

        paragraph
    }
    
    pub fn send<S: AsRef<str>>(&mut self, message: S) -> Result<(), String> {
        match self.sender.send(message.as_ref().to_owned()){
            Ok(_) => Ok(()),
            Err(err) => Err(err.to_string())
        }
    }
    
    pub fn send_line<S: AsRef<str>>(&mut self, message: S) -> Result<(), String> {
        self.send(message.as_ref().to_owned() + "\n")
    }

    pub fn recv(&mut self){
        if let Ok(output) = self.receiver.try_recv(){
            self.output += &output;
        }
    }
}
