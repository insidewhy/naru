extern crate pancurses;
extern crate libc;
use std::io;
use std::io::Write;
use std::fs::File;
use std::os::unix::io::FromRawFd;
use std::ffi::CString;
use std::process::exit;
use pancurses::*;


const COLOR_TABLE: [i16; 8] = [
    COLOR_RED,
    COLOR_BLUE,
    COLOR_GREEN,
    COLOR_CYAN,
    COLOR_RED,
    COLOR_MAGENTA,
    COLOR_YELLOW,
    COLOR_WHITE
];

fn main() {
  let mut choices: Vec<String> = Vec::new();
  let mut input = String::new();
  loop {
    match io::stdin().read_line(&mut input) {
      Ok(n) => {
        if n == 0 {
          break;
        }
        choices.push(input.trim().clone().to_string());
        input.clear();
      }
      Err(error) => println!("error: {}", error),
    }
  }

  let stdout_fd;
  unsafe {
    let var = libc::fopen(
      CString::new("/dev/tty").unwrap().as_ptr(),
      CString::new("r+").unwrap().as_ptr()
      );
    stdout_fd = libc::dup(1);
    libc::dup2(libc::fileno(var), 0);
    libc::dup2(libc::fileno(var), 1);
  }

  let window = initscr();
  window.refresh();
  window.keypad(true);
  if has_colors() {
    start_color();
    use_default_colors();
  }

  for (i, color) in COLOR_TABLE.into_iter().enumerate() {
    init_pair(i as i16, *color, COLOR_WHITE);
  }

  window.nodelay(false);
  noecho();

  let mut idx: i32 = 0;
  for (_idx, choice) in choices.iter().enumerate() {
    if idx == _idx as i32 {
      window.color_set(COLOR_MAGENTA);
    }
    else {
      window.color_set(0);
    }
    window.printw(&choice);
    window.printw("\n");
  }

  loop {
    match window.getch() {
      Some(val) => {
        match val {
          Input::Character('n') | Input::Character('j') => {
            idx += 1;
            let n_choices = choices.len() as i32;
            if idx >= n_choices {
              idx = n_choices - 1;
            }
            else {
              if idx > 0 {
                window.color_set(0);
                window.mvprintw(idx - 1, 0 as i32, &choices[(idx - 1) as usize]);
              }
              window.color_set(COLOR_MAGENTA);
              window.mvprintw(idx, 0 as i32, &choices[idx as usize]);
            }
          },
          Input::Character('e') | Input::Character('k') => {
            idx -= 1;
            if idx < 0 {
              idx = 0;
            }
            else {
              let n_choices = choices.len() as i32;
              if idx < n_choices - 1 {
                window.color_set(0);
                window.mvprintw(idx + 1, 0 as i32, &choices[(idx + 1) as usize]);
              }
              window.color_set(COLOR_MAGENTA);
              window.mvprintw(idx, 0 as i32, &choices[idx as usize]);
            }
          },
          Input::KeyEnter | Input::Character('\n') | Input::Character('c') => {
            break;
          },
          _ => break,
        }
      },
      None => (),
    }
  }
  endwin();

  let mut file = unsafe { File::from_raw_fd(stdout_fd) };
  match file.write_all(choices[idx as usize].as_bytes()) {
    Ok(_) => exit(0),
    Err(_) => exit(1),
  }
}
