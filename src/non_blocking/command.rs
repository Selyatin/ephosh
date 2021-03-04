use std::convert::AsRef;
use std::process::{self, Child, ChildStderr, ChildStdin, ChildStdout, Stdio};
use std::thread;

use std::io::{Read, Write};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{channel, Sender},
    Arc, Mutex,
};

#[derive(Debug, Clone)]
pub struct Command {
    args: Vec<String>,
    command: String,
    output: Arc<Mutex<Vec<u8>>>,
    kill: Arc<AtomicBool>,
    stdin_sender: Option<Sender<char>>,
}

impl Command {
    pub fn new<S: AsRef<str>>(command_name: S) -> Self {
        Self {
            args: vec![],
            command: command_name.as_ref().to_owned(),
            output: Arc::new(Mutex::new(vec![])),
            kill: Arc::new(AtomicBool::new(false)),
            stdin_sender: None,
        }
    }

    pub fn arg<S: AsRef<str>>(&mut self, arg: S) -> &mut Self {
        self.args.push(arg.as_ref().to_owned());

        self
    }

    pub fn args<T: IntoIterator<Item = S>, S: AsRef<str>>(&mut self, collection: T) -> &mut Self {
        for arg in collection.into_iter() {
            self.arg(arg);
        }

        self
    }

    pub fn kill_process(&self) {
        self.kill.store(true, Ordering::Relaxed);
    }

    pub fn send_char(&mut self, c: char) -> Result<(), String> {
        if let Some(sender) = &self.stdin_sender {
            if let Err(err) = sender.send(c) {
                return Err(err.to_string());
            }
        }
        Ok(())
    }

    pub fn get_output(&mut self) -> Result<Vec<u8>, String> {
        let output_lock = match self.output.lock() {
            Ok(lock) => lock,
            Err(err) => return Err(err.to_string()),
        };
        let clone = output_lock.clone();

        drop(output_lock);

        Ok(clone)
    }

    fn output_reader(&self, mut stdout: ChildStdout, mut stderr: ChildStderr) {
        let output_clone = self.output.clone();
        let kill_clone = self.kill.clone();

        thread::spawn(move || loop {
            if kill_clone.load(Ordering::Relaxed) {
                break;
            }

            let mut buff = [0 as u8; 5048];

            match stdout.read(&mut buff) {
                Ok(size) => {
                    if size > 0 {
                        let mut output_vec = match output_clone.lock() {
                            Ok(lock) => lock,
                            Err(err) => panic!("Error: {}", err.to_string()),
                        };

                        output_vec.extend(buff.iter().cloned());
                    } else {
                        break;
                    }
                }
                Err(err) => panic!("Error: {}", err.to_string()),
            };

            drop(buff);
        });

        let output_clone = self.output.clone();
        let kill_clone = self.kill.clone();

        thread::spawn(move || loop {
            if kill_clone.load(Ordering::Relaxed) {
                break;
            }

            let mut buff = [0 as u8; 5048];
            match stderr.read(&mut buff) {
                Ok(size) => {
                    if size > 0 {
                        let mut output_vec = match output_clone.lock() {
                            Ok(lock) => lock,
                            Err(err) => panic!("Error: {}", err.to_string()),
                        };

                        output_vec.extend(buff.iter().cloned());
                    }
                }
                Err(err) => panic!("Error: {}", err.to_string()),
            };

            drop(buff);
        });
    }

    fn input_reader(&self, mut stdin: ChildStdin) -> Sender<char> {
        let kill_clone = self.kill.clone();

        let (sender, receiver) = channel::<char>();

        thread::spawn(move || loop {
            if kill_clone.load(Ordering::Relaxed) {
                break;
            }

            if let Ok(c) = receiver.recv() {
                if stdin.write(c.to_string().as_bytes()).is_err() {
                    break;
                }
            }
        });

        sender
    }

    /// Waits for the child process to finish, it'll remove it from the process table of the OS
    fn process_waiter(&self, mut child: Child) {
        let kill_clone = self.kill.clone();
        thread::spawn(move || {
            if let Ok(_) = child.wait() {
                kill_clone.store(true, Ordering::Relaxed)
            }
        });
    }

    pub fn spawn(&mut self) -> Result<(), String> {
        let mut process = match process::Command::new(&self.command)
            .args(&self.args)
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(process) => process,
            Err(err) => return Err(err.to_string()),
        };

        let stdin = process.stdin.take().unwrap();
        let stdout = process.stdout.take().unwrap();
        let stderr = process.stderr.take().unwrap();

        self.process_waiter(process);

        self.output_reader(stdout, stderr);

        let sender_input = self.input_reader(stdin);

        self.stdin_sender = Some(sender_input);

        Ok(())
    }
}
