use crate::{config::Config, other_error, tty::Tty};

use std::{
  collections::HashMap,
  ffi::CStr,
  io,
  io::{Error, ErrorKind},
};

macro_rules! def_action_names {
  ($actions_by_name: ident, $($name: expr => $mapping: ident);+;) => {
    $(
      $actions_by_name.insert($name.to_string(), Self::$mapping);
    )+
  };
}

macro_rules! def_default_mappings {
  ($actions: ident, $($name: expr => $mapping: ident);+;) => {{
    $(
      let name_str = $name.to_string();
      if ! $actions.contains_key(&name_str) {
        $actions.insert(name_str, Self::$mapping);
      }
    )+
  }};
}

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

    let actions = Self::build_actions(&self.conf.bindings)?;
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
              let action = actions.get(input);
              if action.is_some() {
                action.unwrap()(self)?;
              }
            } else {
              self.criteria.push_str(input);
              self.terminal.print(input)?;
              self.terminal.flush();
            }
          }
          Err(_) => {
            return other_error!("Could not convert string");
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

  fn build_actions(
    bindings: &HashMap<String, String>,
  ) -> io::Result<HashMap<String, fn(&mut Self) -> io::Result<()>>> {
    let mut actions_by_name: HashMap<String, fn(&mut Self) -> io::Result<()>> = HashMap::new();
    def_action_names!(
      actions_by_name,
      "select-prev" => select_prev;
      "select-next" => select_next;
    );

    let mut actions: HashMap<_, fn(&mut Self) -> io::Result<()>> = HashMap::new();
    for (a, b) in bindings {
      let action = actions_by_name.get(b);
      if !action.is_some() {
        return other_error!(format!("Invalid action name {}", b));
      }

      actions.insert(a.clone(), *action.unwrap());
    }

    def_default_mappings!(
      actions,
      "\x1b[A" => select_prev;
      "\x1bOA" => select_prev;
      "\x1b[B" => select_next;
      "\x1bOB" => select_next;
    );
    Ok(actions)
  }

  fn select_next(selector: &mut Self) -> io::Result<()> {
    if selector.selected + 1 < selector.choices.len() {
      selector.selected += 1;
      selector.redraw()?;
    }
    Ok(())
  }

  fn select_prev(selector: &mut Self) -> io::Result<()> {
    if selector.selected > 0 {
      selector.selected -= 1;
      selector.redraw()?;
    }
    Ok(())
  }
}