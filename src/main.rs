extern crate libc;
mod config;
mod tty;
use std::io;
use config::{Config, load_config};
use tty::Tty;

const TTY_PATH: &str = "/dev/tty";

fn match_input(conf: &Config) -> io::Result<()> {
  let mut choices: Vec<String> = Vec::new();
  let mut input = String::new();
  loop {
    let n = io::stdin().read_line(&mut input)?;
    if n == 0 {
      break;
    }
    choices.push(input.trim().to_string());
    input.clear();
  }

  let terminal = Tty::new(&TTY_PATH)?;

  // TODO: enable filtering etc.
  // for choice in choices { println!("{}", choice); }

  terminal.reset()
}

fn main() {
  let conf = load_config().unwrap();
  println!("{}", conf.window.height);
  match_input(&conf).unwrap();
}
