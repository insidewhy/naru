use crate::{config::Config, control_key, other_error, tty, tty::Tty};
use sublime_fuzzy::best_match;

use std::{collections::HashMap, ffi::CStr, io};

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
      $actions.entry($name.to_string()).or_insert(Self::$mapping);
    )+
  }};
}

struct Match<'a> {
  // match ranges
  choice: &'a String,
  ranges: Vec<(usize, usize)>,
  score: isize,
}

pub(crate) struct Selector<'a, 'b> {
  // inputs
  terminal: &'a mut Tty,
  choices: &'b Vec<String>,
  matches: Vec<Match<'b>>,
  conf: &'b Config,

  // min of terminal height or config height
  height: usize,
  selected: usize,
  criteria: String,

  // first visible choice, used like a sliding window the user pushes around
  first_visible_option_idx: usize,
}

impl<'a, 'b> Selector<'a, 'b> {
  pub fn new(
    terminal: &'a mut Tty,
    choices: &'b Vec<String>,
    conf: &'b Config,
  ) -> Selector<'a, 'b> {
    let max_height = terminal.max_height as usize;
    let height = if conf.window.height > 0 {
      std::cmp::min(conf.window.height as usize, max_height)
    } else {
      std::cmp::max(max_height + (conf.window.height as usize), 1)
    };

    Selector {
      terminal,
      choices,
      matches: Vec::new(),
      conf,
      height,
      selected: 0,
      criteria: String::new(),
      first_visible_option_idx: 0,
    }
  }

  pub fn get_match(&mut self) -> io::Result<&'b String> {
    self.draw_options()?;
    self.terminal.flush();

    let actions = Self::build_actions(&self.conf.bindings)?;
    let input_reader = self.terminal.get_reader();

    loop {
      let data = input_reader.read()?;
      if data[0] == 0 {
        // signal interrupt, redraw screen in case it was WINCH
        self.redraw()?;
        continue;
      }
      let str_ptr_result = unsafe { CStr::from_ptr(data.as_ptr() as *mut i8) }.to_str();
      match str_ptr_result {
        Ok(input) => {
          if input == "\r" {
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
            self.update_matches()?;
          }
        }
        Err(_) => {
          return other_error!("Could not convert string");
        }
      }
    }

    if self.criteria.len() == 0 {
      Ok(&self.choices[self.selected])
    } else {
      Ok(self.matches[self.selected].choice)
    }
  }

  fn redraw(&mut self) -> io::Result<()> {
    self.draw_options()?;
    self.terminal.print(&self.criteria)?;
    self.terminal.flush();
    Ok(())
  }

  // draw choices if there are no criteria, otherwise draw matches
  fn draw_options(&mut self) -> io::Result<()> {
    let has_criteria = self.criteria.len() != 0;
    let option_count = if has_criteria {
      self.matches.len()
    } else {
      self.choices.len()
    };

    let visible_option_count = std::cmp::min(self.height - 1, option_count);
    if self.selected >= self.first_visible_option_idx + visible_option_count {
      self.first_visible_option_idx = self.selected + 1 - visible_option_count;
    } else if self.selected < self.first_visible_option_idx {
      self.first_visible_option_idx = self.selected;
    }

    if has_criteria {
      self.draw_matches(visible_option_count)?;
    } else {
      self.draw_choices(visible_option_count)?;
    }

    // move to the "top"
    self.terminal.set_normal()?;
    self.terminal.set_col(0)?;
    self.terminal.print("> ")?;
    self.terminal.clearline()?;

    Ok(())
  }

  fn draw_choices(&mut self, visible_option_count: usize) -> io::Result<()> {
    for line_idx in 0..visible_option_count {
      self.terminal.newline()?;
      let choice_idx = line_idx + self.first_visible_option_idx;
      let choice = &self.choices[choice_idx];

      if choice_idx == self.selected {
        // this ensures that the invert sgr is not cleared by a reset byte
        let last_sgr_byte = tty::find_last_sgr_byte(choice.as_bytes());
        if last_sgr_byte != 0 {
          self.terminal.print(&choice[0..last_sgr_byte])?;
          self.terminal.print(";7")?;
          self.terminal.print(&choice[last_sgr_byte..])?;
        } else {
          self.terminal.set_invert()?;
          self.terminal.print(choice)?;
        }
        self.terminal.set_normal()?;
      } else {
        self.terminal.print(choice)?;
      }
    }

    self.terminal.clearline()?;
    self.terminal.move_up(visible_option_count as i32)?;

    Ok(())
  }

  fn draw_matches(&mut self, visible_option_count: usize) -> io::Result<()> {
    for line_idx in 0..visible_option_count {
      self.terminal.newline()?;
      let match_idx = line_idx + self.first_visible_option_idx;
      let thismatch = &self.matches[match_idx];
      let choice = thismatch.choice;

      let is_selected = match_idx == self.selected;

      let last_sgr_byte = tty::find_last_sgr_byte(choice.as_bytes());
      if last_sgr_byte != 0 {
        self.terminal.print(&choice[0..last_sgr_byte])?;
        if is_selected {
          self.terminal.print(";7")?;
        }
      } else if is_selected {
        self.terminal.set_invert()?;
      }

      let mut last_range_end = last_sgr_byte;
      for range in &thismatch.ranges {
        if last_range_end < range.0 {
          // print text before the match
          self.terminal.print(&choice[last_range_end..range.0])?;
        }
        self.terminal.set_fg(5)?;
        let range_end = range.0 + range.1;
        self.terminal.print(&choice[range.0..range_end])?;
        self.terminal.set_normal()?;
        if last_sgr_byte != 0 {
          self.terminal.print(&choice[0..last_sgr_byte])?;
          if is_selected {
            self.terminal.print(";7m")?;
          } else {
            self.terminal.print("m")?;
          }
        } else if is_selected {
          self.terminal.set_invert()?;
        }
        last_range_end = range_end;
      }
      if last_range_end < choice.len() {
        self.terminal.print(&choice[last_range_end..choice.len()])?;
      }

      if is_selected {
        self.terminal.set_normal()?;
      }
    }

    let screen_height = std::cmp::min(self.height - 1, self.choices.len());
    for _ in visible_option_count..screen_height {
      self.terminal.newline()?;
    }

    self.terminal.clearline()?;
    self.terminal.move_up(screen_height as i32)?;

    Ok(())
  }

  fn update_matches(&mut self) -> io::Result<()> {
    let mut matches: Vec<Match<'b>> = self
      .choices
      // TODO: use par_iter from rayon
      .iter()
      .map(|choice| match best_match(&self.criteria, choice) {
        None => None,
        Some(v) => Some(Match {
          ranges: v.continuous_matches(),
          score: v.score(),
          choice: &choice,
        }),
      })
      .flatten()
      .collect();

    matches.sort_by(|a, b| b.score.cmp(&a.score));
    self.selected = 0;
    self.matches = matches;
    self.redraw()
  }

  fn build_actions(
    bindings: &HashMap<String, String>,
  ) -> io::Result<HashMap<String, fn(&mut Self) -> io::Result<()>>> {
    let mut actions_by_name: HashMap<String, fn(&mut Self) -> io::Result<()>> = HashMap::new();
    def_action_names!(
      actions_by_name,
      "select-prev" => select_prev;
      "select-next" => select_next;
      "backspace" => backspace;
    );

    let mut actions: HashMap<_, fn(&mut Self) -> io::Result<()>> = HashMap::new();
    for (a, b) in bindings {
      let action = actions_by_name.get(b);
      if !action.is_some() {
        return other_error!(format!("Invalid action name '{}'", b));
      }

      actions.insert(a.clone(), *action.unwrap());
    }

    def_default_mappings!(
      actions,
      tty::KEY_UP => select_prev;
      tty::KEY_UP_ALTERNATE => select_prev;
      control_key!(b'k') => select_prev;
      control_key!(b'e') => select_prev;
      tty::KEY_DOWN => select_next;
      tty::KEY_DOWN_ALTERNATE => select_next;
      control_key!(b'j') => select_next;
      control_key!(b'n') => select_next;

      control_key!(b'h') => backspace;
      "\x7f" => backspace;
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

  fn backspace(selector: &mut Self) -> io::Result<()> {
    if selector.criteria.len() != 0 {
      selector.criteria.pop();
      selector.update_matches()?;
      selector.redraw()?;
    }
    Ok(())
  }
}
