extern crate libc;
use std::io;
use std::process::exit;
mod tty;

fn main() {
  let mut choices: Vec<String> = Vec::new();
  let mut input = String::new();
  loop {
    match io::stdin().read_line(&mut input) {
      Ok(n) => {
        if n == 0 {
          break;
        }
        choices.push(input.trim().to_string());
        input.clear();
      }
      Err(error) => println!("error: {}", error),
    }
  }

  let tty_path = "/dev/tty";
  let terminal = match tty::Tty::init(& tty_path) {
    Ok(t) => t,
    Err(_) => panic!("Could not initialise terminal")
  };

  // TODO: enable filtering etc.
  // for choice in choices {
  //   println!("{}", choice);
  // }

  terminal.reset();
  exit(0);
}
