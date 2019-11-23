mod config;
mod tty;
use config::{load_config, Config};
use std::ffi::CStr;
use std::{
  io,
  io::{Error, ErrorKind},
};
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

  Ok(())
}

fn selector<'a>(
  mut terminal: &mut Tty,
  conf: &Config,
  choices: &'a Vec<String>,
) -> io::Result<&'a String> {
  let height = std::cmp::min(conf.window.height, terminal.max_height);

  // index of selected item
  let mut selected = 0;
  let mut criteria = String::new();

  draw_matches(&mut terminal, &choices, height, selected)?;
  terminal.flush();

  let input_reader = terminal.get_reader();

  loop {
    let data = input_reader.read()?;
    if data[0] == 0 {
      // signal interrupt, redraw screen in case it was WINCH
      draw_matches(&mut terminal, &choices, height, selected)?;
      terminal.print(&criteria)?;
      terminal.flush();
    } else {
      let str_ptr_result = unsafe { CStr::from_ptr(data.as_ptr() as *mut i8) }.to_str();
      match str_ptr_result {
        Ok(input) => {
          if input == "\r" || input == "\n" {
            break;
          }

          let mut chars = input.chars();
          let first_char = chars.next();
          if first_char.is_none() {
            continue;
          }

          if first_char.unwrap().is_ascii_control() {
            // TODO: lookup mapping in configuration, remove hard-coded mappings
            if input == "\x1b[A" || input == "\x1bOA" {
              if selected > 0 {
                selected -= 1;
                draw_matches(&mut terminal, &choices, height, selected)?;
                terminal.print(&criteria)?;
                terminal.flush();
              }
            } else if input == "\x1b[B" || input == "\x1bOB" {
              if selected + 1 < choices.len() {
                selected += 1;
                draw_matches(&mut terminal, &choices, height, selected)?;
                terminal.print(&criteria)?;
                terminal.flush();
              }
            }
          } else {
            criteria.push_str(input);
            terminal.print(input)?;
            terminal.flush();
          }
        }
        Err(_) => {
          return Err(Error::new(ErrorKind::Other, "Could not convert string"));
        }
      }
    }
  }

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

fn main() -> io::Result<()> {
  let conf = load_config()?;
  match_input(&conf)?;
  Ok(())
}
