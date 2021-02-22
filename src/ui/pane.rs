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
            receiver,
            sender
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

    pub fn recv(&mut self){
        if let Ok(output) = self.receiver.try_recv(){
            self.output += &output;
        }
    }
}
