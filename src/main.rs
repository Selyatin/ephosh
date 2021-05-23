mod shell;
mod command;

use shell::Shell;

fn main(){
    let mut shell = Shell::new();
    
    shell.run().expect("Couldn't run shell properly"); 
}
