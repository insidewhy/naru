use crate::{config::Config, tty::Tty};

use std::{
  ffi::CStr,
  io,
  io::{Error, ErrorKind},
};

pub(crate) struct Selector<'a, 'b> {
  // inputs
  terminal: &'a mut Tty,
  choices: &'b Vec<String>,
  conf: &'b Config,

  // min of terminal height or config height
  height: u16,
  selected: usize,
  criteria: String,
}

impl<'a, 'b> Selector<'a, 'b> {
  pub fn new(
    terminal: &'a mut Tty,
    choices: &'b Vec<String>,
    conf: &'b Config,
  ) -> Selector<'a, 'b> {
    let max_height = terminal.max_height;

    Selector {
      terminal,
      choices,
      conf,
      height: std::cmp::min(conf.window.height, max_height),
      selected: 0,
      criteria: String::new(),
    }
  }

  pub fn get_match(&mut self) -> io::Result<&'b String> {
    self.draw_matches()?;
    self.terminal.flush();

    let input_reader = self.terminal.get_reader();

    loop {
      let data = input_reader.read()?;
      if data[0] == 0 {
        // signal interrupt, redraw screen in case it was WINCH
        self.redraw()?;
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
                if self.selected > 0 {
                  self.selected -= 1;
                  self.redraw()?;
                }
              } else if input == "\x1b[B" || input == "\x1bOB" {
                if self.selected + 1 < self.choices.len() {
                  self.selected += 1;
                  self.redraw()?;
                }
              }
            } else {
              self.criteria.push_str(input);
              self.terminal.print(input)?;
              self.terminal.flush();
            }
          }
          Err(_) => {
            return Err(Error::new(ErrorKind::Other, "Could not convert string"));
          }
        }
      }
    }

    Ok(&self.choices[self.selected])
  }

  fn redraw(&mut self) -> io::Result<()> {
    self.draw_matches()?;
    self.terminal.print(&self.criteria)?;
    self.terminal.flush();
    Ok(())
  }

  fn draw_matches(&mut self) -> io::Result<()> {
    let line_count = std::cmp::min(self.height as usize, self.choices.len() + 1);

    for line_idx in 0..line_count - 1 {
      self.terminal.newline()?;

      if line_idx == self.selected {
        self.terminal.set_invert()?;
      }

      self.terminal.print(self.choices[line_idx].as_str())?;

      if line_idx == self.selected {
        self.terminal.set_normal()?;
      }
    }

    // move to the "top"
    self.terminal.move_up((line_count - 1) as i32)?;
    self.terminal.set_normal()?;
    self.terminal.set_col(0)?;
    self.terminal.print("> ")?;
    self.terminal.clearline()?;

    Ok(())
  }
}
