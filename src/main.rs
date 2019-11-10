extern crate libc;
extern crate xdg;
use std::io;
mod tty;

const TTY_PATH: &str = "/dev/tty";

fn load_config() -> io::Result<()> {
  let xdg_dirs = xdg::BaseDirectories::new()?;
  let cfg_file = xdg_dirs.find_config_file("toss.toml");

  match cfg_file {
    // TODO: return default config
    None => println!("no config"),
    // TODO: parse config file
    Some(v) => println!("{}", v.to_str().unwrap()),
  };

  Ok(())
}

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

  let terminal = tty::Tty::init(&TTY_PATH)?;

  // TODO: enable filtering etc.
  // for choice in choices { println!("{}", choice); }

  terminal.reset()
}

fn main() {
  load_config().unwrap();
  match_input().unwrap();
}
