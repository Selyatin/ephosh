use std::process::{self, Stdio};
use std::convert::AsRef;
use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::os::unix::io::AsRawFd;
use std::io::{self, Read, Write, BufRead, BufReader};

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
            self.arg(arg);
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
        let (sender_input, _receiver_input) = channel::<String>();

        thread::spawn(move || {
            let mut process = process_result.unwrap();

            let _stdin = process.stdin.take().unwrap();
            let mut stdout = process.stdout.take().unwrap();
            let mut stderr = process.stderr.take().unwrap();

            let mut buffer = [0 as u8; 10024];

            loop {

                match stdout.read(&mut buffer){
                    Ok(size) => {
                        if size <= 0 {
                            break;
                        }

                        if sender_output.send(String::from_utf8_lossy(&buffer).to_string()).is_err(){
                            break;   
                        }
                    },

                    Err(_) => {
                        break;
                    }
                };

            }

            loop {

                match stderr.read(&mut buffer){
                    Ok(size) => {
                        if size <= 0 {
                            break;
                        }

                        if sender_output.send(String::from_utf8_lossy(&buffer).to_string()).is_err(){
                            break;
                        }
                    },

                    Err(_) => {
                        break;
                    }
                };

            }
        });

        Ok((sender_input, receiver_output))
    }
}
