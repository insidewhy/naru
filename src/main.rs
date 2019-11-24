mod config;
mod selector;
mod tty;
use config::{load_config, Config};
use selector::Selector;
use std::io;
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
  let conf = load_config()?;
  match_input(&conf)?;
  Ok(())
}
