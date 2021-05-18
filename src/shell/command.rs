use std::{
    thread,
    time::Duration,
    io::{
        Read,
        Write
    },
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

const BUFFER_SIZE: usize = 32162;

pub struct Command {
    bytes: [u8; BUFFER_SIZE],
    kill: Arc<AtomicBool>,
    reader: Box<dyn Read + Send>,
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

        let mut writer = match pair.master.try_clone_writer(){
            Ok(writer) => writer,
            Err(err) => return Err(err.to_string())
        };
        
        let reader = match pair.master.try_clone_reader(){
            Ok(reader) => reader,
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
                thread::sleep(Duration::from_millis(100));
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
            reader,
            stdin_channel: sender,
            bytes: [0 as u8; BUFFER_SIZE]
        })
    }

    pub fn resize(&mut self, terminal_size: (u16, u16)) -> Result<(), String>{
        if let Err(err) = self.pair.master.resize(PtySize{
            cols: terminal_size.0,
            rows: terminal_size.1,
            pixel_width: 0,
            pixel_height: 0
        }){
            return Err(err.to_string());
        }

        Ok(())
    }

    pub fn get_output<'a>(&mut self) -> Result<&[u8], String> {
        if let Err(err) = self.reader.read(&mut self.bytes) {
            return Err(err.to_string())
        };

        Ok(&self.bytes)
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
