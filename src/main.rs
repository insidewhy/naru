extern crate libc;
mod config;
mod tty;
use std::io;
use config::{Config, load_config};
use tty::Tty;

const TTY_PATH: &str = "/dev/tty";

fn run_matcher(terminal: &mut Tty, conf: &Config, choices: &Vec<String>) -> io::Result<()> {
  // TODO: enable filtering etc.
  // for choice in choices { println!("{}", choice); }
  for i in 0..8 {
    terminal.set_fg(i)?;
    terminal.putc(64 + i);
  }
  terminal.newline()?;
  println!("TODO: get match");
  Ok(())
}

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

  let mut terminal = Tty::new(&TTY_PATH)?;
  let result = run_matcher(&mut terminal, &conf, &choices);
  terminal.reset();
  result
}

fn main() {
  let conf = load_config().unwrap();
  match_input(&conf).unwrap();
}
