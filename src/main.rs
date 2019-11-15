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
  selected: usize,
) -> io::Result<()> {
  let line_count = std::cmp::min(height as usize, choices.len() + 1);

  for line_idx in 0..line_count - 1 {
    terminal.newline()?;

    if line_idx == selected {
      terminal.set_invert()?;
    }

    terminal.print(choices[line_idx].as_str())?;

    if line_idx == selected {
      terminal.set_normal()?;
    }
  }

  // move to the "top"
  terminal.move_up((line_count - 1) as i32)?;
  terminal.set_normal()?;
  terminal.set_col(0)?;
  terminal.print("> ")?;
  terminal.clearline()?;

  terminal.flush();

  Ok(())
}

fn selector<'a>(
  mut terminal: &mut Tty,
  conf: &Config,
  choices: &'a Vec<String>,
) -> io::Result<&'a String> {
  let height = std::cmp::min(conf.window.height, terminal.max_height);

  let mut selected = 0;
  draw_matches(&mut terminal, &choices, height, selected)?;

  // TODO: wait for input etc.
  std::thread::sleep(std::time::Duration::from_secs(1));

  Ok(&choices[selected])
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
  terminal.set_no_wrap()?;

  let result = selector(&mut terminal, &conf, &choices);

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

fn main() {
  let conf = load_config().unwrap();
  match_input(&conf).unwrap();
}
