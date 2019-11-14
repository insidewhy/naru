extern crate libc;
mod config;
mod tty;
use config::{load_config, Config};
use std::io;
use tty::Tty;

const TTY_PATH: &str = "/dev/tty";

fn draw_matches(
  terminal: &mut Tty,
  choices: &Vec<String>,
  height: u16,
  selected: i32,
) -> io::Result<()> {
  for line in 0..height - 1 {
    terminal.newline()?;

    if line == (selected as u16) {
      terminal.set_invert()?;
    }

    // TODO: enable filtering etc.
    // for choice in choices { println!("{}", choice); }
    for i in 0..8 {
      terminal.set_fg(i)?;
      terminal.putc(64 + i);
    }

    if line == (selected as u16) {
      terminal.set_normal()?;
    }
  }

  // move to the "top"
  terminal.move_up((height - 1) as i32)?;
  terminal.set_normal()?;
  terminal.set_col(0)?;
  terminal.print("> ")?;
  terminal.clearline()?;

  terminal.flush();

  Ok(())
}

fn selector(mut terminal: &mut Tty, conf: &Config, choices: &Vec<String>) -> io::Result<()> {
  let height = std::cmp::min(conf.window.height, terminal.max_height);

  let mut selected = 0;
  draw_matches(&mut terminal, &choices, height, selected)?;

  // TODO: wait for input etc.
  std::thread::sleep(std::time::Duration::from_secs(1));

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
  let result = selector(&mut terminal, &conf, &choices);

  terminal.set_col(0)?;
  terminal.clearline()?;
  terminal.set_normal()?;
  terminal.reset();
  println!("TODO: get match");
  result
}

fn main() {
  let conf = load_config().unwrap();
  match_input(&conf).unwrap();
}
