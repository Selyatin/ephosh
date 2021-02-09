use std::process::{self, Stdio};
use std::convert::AsRef;
use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::io::{self, Read, Write};

#[derive(Debug, Clone)]
pub struct Command {
    args: Vec<String>,
    command: String
}

impl Command<> {

    pub fn new<S: AsRef<str>>(command_name: S) -> Self {
        Self {
            args: vec![],
            command: command_name.as_ref().to_owned()
        }
    }

    pub fn arg<S: AsRef<str>>(&mut self, arg: S) -> &mut Self{
        self.args.push(arg.as_ref().to_owned());

        self
    }

    pub fn args<T: IntoIterator<Item = S>, S: AsRef<str>>(&mut self, collection: T) -> &mut Self{
        for arg in collection.into_iter(){
            self.args.push(arg.as_ref().to_owned());
        }

        self
    }

    pub fn spawn(&self) -> io::Result<(Sender<String>, Receiver<String>)>{
        let process_result = process::Command::new(&self.command)
            .args(&self.args)
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        if let Err(err) = process_result {
            return Err(err);
        }

        let (sender_output, receiver_output) = channel::<String>();
        let (sender_input, receiver_input) = channel::<String>();

        thread::spawn(move || {
            let mut process = process_result.unwrap();

            let mut stdin = process.stdin.take().unwrap();
            let mut stdout = process.stdout.take().unwrap();
            let mut stderr = process.stderr.take().unwrap();

            loop {

                // Check if process has exited without blocking the thread.
                if let Ok(Some(_)) = process.try_wait() {
                    break;
                }

                
                let mut output = String::new();

                if let Ok(input) = receiver_input.try_recv() {
                    if let Err(err) = stdin.write_all(input.as_bytes()){
                        sender_output.send(err.to_string()).unwrap();
                        continue;
                    }
                }

                
                if let Err(err) = stdout.read_to_string(&mut output){
                    sender_output.send(err.to_string()).unwrap();
                    continue;
                }

                if let Err(err) = stderr.read_to_string(&mut output){
                    sender_output.send(err.to_string()).unwrap();
                    continue;
                }

                if let Err(_) = sender_output.send(output){
                    break;
                }



            }
        });
        Ok((sender_input, receiver_output))
    }
}
