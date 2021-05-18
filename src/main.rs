use std::{
    cmp::Ordering,
    env,
    io::{self, Write},
};

mod config;
mod inbuilt;
mod shell;
mod ui;
mod utils;

use shell::Shell;
use ui::{
    input::{self, InputMode},
    Pane,
};

fn main() {
    let mut shell = Shell::default();

}
