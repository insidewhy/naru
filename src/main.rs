mod c_str;
mod config;
mod selector;
mod tty;
use config::{load_config, Config};
use selector::Selector;
use std::{error::Error, io, io::ErrorKind};
use tty::Tty;

#[macro_export]
macro_rules! other_error {
  ($message: expr) => {
    Err(std::io::Error::new(std::io::ErrorKind::Other, $message))
  };

  ($($message: expr),+) => {
    Err(std::io::Error::new(std::io::ErrorKind::Other, format!( $($message,)+ )))
  };
}

const TTY_PATH: &str = "/dev/tty";

fn match_input(conf: &Config) -> io::Result<()> {
  let mut choices: Vec<String> = Vec::new();
  let mut input = String::new();
  loop {
    let n = io::stdin().read_line(&mut input)?;
    if n == 0 {
      break;
    }
    let trimmed = input.trim();
    if trimmed.len() != 0 {
      choices.push(trimmed.to_string());
    }
    input.clear();
  }

  let mut terminal = Tty::new(&TTY_PATH)?;
  terminal.set_no_wrap()?;

  let result = {
    let mut selector = Selector::new(&mut terminal, &choices, &conf);
    selector.get_match()
  };

  terminal.set_wrap()?;
  terminal.set_col(0)?;
  terminal.clearline()?;
  terminal.set_normal()?;
  terminal.reset();
  result.map(|selected| {
    println!("{}", selected);
    ()
  })
}

fn main() -> io::Result<()> {
  let result = {
    let conf = load_config()?;
    match_input(&conf)
  };

  match result {
    Err(ref e) if e.kind() == ErrorKind::Other => {
      eprintln!("{}", e.description());
      Ok(())
    }
    default => default,
  }
}
