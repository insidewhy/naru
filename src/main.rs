extern crate libc;
use std::io;
mod tty;

fn match_input() -> io::Result<()> {
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

  let tty_path = "/dev/tty";
  let terminal = tty::Tty::init(& tty_path)?;

  // TODO: enable filtering etc.
  // for choice in choices {
  //   println!("{}", choice);
  // }

  terminal.reset()
}

fn main() {
  match_input().unwrap();
}
