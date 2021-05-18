use std::{
    thread,
    time::Duration,
    sync::{
        mpsc::{
            channel,
            Sender,
            Receiver
        },
        Arc,
        Mutex,
        atomic::{AtomicBool, Ordering}
    }
};
use portable_pty::{
    PtySystem,
    PtyPair,
    PtySize,
    Child,
    CommandBuilder
};

pub struct Command {
    bytes: Arc<Mutex<Vec<u8>>>,
    kill: Arc<AtomicBool>,
    stdin_channel: Sender<char>,
    pair: PtyPair
}

impl Command {
    pub fn new<'a>(pty: &Box<dyn PtySystem>, args: impl IntoIterator<Item = &'a str>, terminal_size: (u16, u16)) -> Result<Self, String> {
        let args: Vec<&str> = args.into_iter().collect();
        if args.len() < 1 {
            return Err("Not enough arguments".to_owned());
        }

        let pair = match pty.openpty(PtySize {
            rows: terminal_size.1,
            cols: terminal_size.0,
            pixel_width: 0,
            pixel_height: 0
        }) {
            Ok(pair) => pair,
            Err(err) => return Err(err.to_string())
        };

        let mut cmd = CommandBuilder::new(args[0]);

        cmd.args(&args[1..]);

        let kill = Arc::new(AtomicBool::new(false));

        let kill_clone = kill.clone();

        let mut child = match pair.slave.spawn_command(cmd) {
            Ok(child) => child,
            Err(err) => return Err(err.to_string())
        };

        let bytes: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(vec![]));

        let mut reader = match pair.master.try_clone_reader(){
            Ok(reader) => reader,
            Err(err) => return Err(err.to_string())
        };

        let mut writer = match pair.master.try_clone_writer(){
            Ok(writer) => writer,
            Err(err) => return Err(err.to_string())
        };

        // Spawn a thread that'll kill the child if commanded to do so
        thread::spawn(move || {
            let kill = kill_clone;
            loop {
                if kill.load(Ordering::Relaxed) {
                    if child.kill().is_err(){}
                    if child.wait().is_err(){}
                }
                thread::sleep(Duration::from_millis(25));
            }
        });

        let bytes_clone = bytes.clone();
        // Spawn a thread that will read the output and save it if there's new output
        thread::spawn(move || {
            let bytes = bytes_clone;

            loop {
                let mut buffer = [0 as u8; 4086];

                match reader.read(&mut buffer) {
                    Ok(size) => {
                        if size < 1 {
                            break;
                        }
                    },
                    Err(_) => break
                };

                let mut lock = match bytes.lock(){
                    Ok(lock) => lock,
                    Err(_) => break
                };
                
                lock.extend(&buffer);
            }
        });

        let (sender, mut receiver) = channel::<char>();

        thread::spawn(move || loop {
            if let Ok(c) = receiver.recv() {
                if writer.write(&[c as u8]).is_err() {
                    break;
                };
            }
        });

        Ok(Self {
            pair,
            kill,
            stdin_channel: sender,
            bytes
        })
}

pub fn resize(&mut self, terminal_size: (u16, u16)) -> Result<(), String>{
    if let Err(err) = self.pair.master.resize(PtySize{
        rows: terminal_size.0,
        cols: terminal_size.1,
        pixel_width: 0,
        pixel_height: 0
    }){
        return Err(err.to_string());
    }

    Ok(())
}

pub fn get_output<'a>(&self) -> Result<Vec<u8>, String> {
    let lock = match self.bytes.lock(){
        Ok(lock) => lock,
        Err(err) => return Err(err.to_string())
    };

    Ok(lock.clone())
}

pub fn send_char(&self, c: char) -> Result<(), String> {
    if let Err(err) = self.stdin_channel.send(c) {
        return Err(err.to_string());
    }

    Ok(())
}

pub fn kill(&mut self){
    self.kill.store(true, Ordering::Relaxed);
}
}
