use libc::{
  c_void, close, fclose, fflush, fileno, fprintf, fputc, ioctl, read, setvbuf, winsize, TIOCGWINSZ,
  _IOFBF,
};
use std::{
  ffi::CString,
  io,
  io::{Error, ErrorKind},
};
use termios::{tcsetattr, Termios, ECHO, ICANON, ICRNL, ISIG, TCSANOW};
#[macro_use]
#[path = "c_str.rs"]
mod c_str;

// Make unsafe call and turn non-zero exit statuses into an io error with the given string when
// needed
macro_rules! fwd_error_code {
  ($expr: expr, $msg: expr) => {
    if unsafe { $expr != 0 } {
      return Err(Error::new(ErrorKind::Other, $msg));
    }
  };
}

macro_rules! terminal_printf {
  ($self: expr, $($expr: expr),+) => {
    if unsafe { fprintf($self.fout, $($expr),+) < 0 } {
      return Err(Error::new(ErrorKind::Other, "Error printing to console"))
    }
  }
}

def_c_str! {
  WRITE_FORMAT = "w";
  CLEAR_LINE_FORMAT = "\x1b[K";
  SGR_FORMAT = "\x1b[%im";
  NEWLINE_FORMAT = "\x1b[K\n";
  SET_COL_FORMAT = "\x1b[%iG";
  NO_WRAP_FORMAT = "\x1b[?7l";
  MOVE_UP_FORMAT = "\x1b[%iA";
  WRAP_FORMAT = "\x1b[?7h";
}

pub(crate) struct Tty {
  fdin: i32,
  fout: *mut libc::FILE,
  original_termios: Termios,
  fg_color: i32,
  pub max_width: u16,
  pub max_height: u16,
}

impl Tty {
  pub fn new(tty_path: &str) -> io::Result<Tty> {
    let tty_filename_c = CString::new(tty_path)?;
    let fdin = unsafe { libc::open(tty_filename_c.as_ptr(), libc::O_RDONLY) };
    let fout = unsafe { libc::fopen(tty_filename_c.as_ptr(), WRITE_FORMAT.as_ptr()) };

    let original_termios = Termios::from_fd(fdin)?;

    let mut termios_copy = original_termios.clone();
    termios_copy.c_iflag &= !(ICRNL);
    termios_copy.c_lflag &= !(ICANON | ECHO | ISIG);
    tcsetattr(fdin, TCSANOW, &termios_copy)?;

    fwd_error_code!(
      setvbuf(fout, std::ptr::null_mut(), _IOFBF, 4096),
      "Could not setvbuf"
    );

    let mut ws = winsize {
      ws_row: 0,
      ws_col: 0,
      ws_xpixel: 0,
      ws_ypixel: 0,
    };
    fwd_error_code!(
      ioctl(fileno(fout), TIOCGWINSZ, &mut ws),
      "Could not get window size"
    );

    Ok(Tty {
      fdin,
      fout,
      fg_color: 9,
      original_termios,
      max_width: ws.ws_col,
      max_height: ws.ws_row,
    })
  }

  pub fn sgr(&self, code: i32) -> io::Result<()> {
    terminal_printf!(self, SGR_FORMAT.as_ptr(), code);
    Ok(())
  }

  pub fn set_fg(&mut self, color: i32) -> io::Result<()> {
    if self.fg_color != color {
      self.sgr(30 + color)?;
      self.fg_color = color;
    }
    Ok(())
  }

  pub fn set_invert(&self) -> io::Result<()> {
    self.sgr(7)
  }

  pub fn set_no_wrap(&self) -> io::Result<()> {
    terminal_printf!(self, NO_WRAP_FORMAT.as_ptr());
    Ok(())
  }

  pub fn set_wrap(&self) -> io::Result<()> {
    terminal_printf!(self, WRAP_FORMAT.as_ptr());
    Ok(())
  }

  // pub fn set_underline(&self) -> io::Result<()> { self.sgr(4) }

  pub fn set_normal(&mut self) -> io::Result<()> {
    self.sgr(0)?;
    self.fg_color = 9;
    Ok(())
  }

  pub fn move_up(&self, row_count: i32) -> io::Result<()> {
    terminal_printf!(self, MOVE_UP_FORMAT.as_ptr(), row_count);
    Ok(())
  }

  pub fn putc(&self, c: i32) {
    unsafe { fputc(c, self.fout) };
  }

  pub fn print(&self, string: &str) -> io::Result<()> {
    terminal_printf!(self, CString::new(string)?.as_ptr());
    Ok(())
  }

  // Remove everything after cursor
  pub fn clearline(&self) -> io::Result<()> {
    terminal_printf!(self, CLEAR_LINE_FORMAT.as_ptr());
    Ok(())
  }

  // Remove everything after cursor then move to next line
  pub fn newline(&self) -> io::Result<()> {
    terminal_printf!(self, NEWLINE_FORMAT.as_ptr());
    Ok(())
  }

  pub fn flush(&self) {
    unsafe { fflush(self.fout) };
  }

  pub fn set_col(&self, col: i32) -> io::Result<()> {
    terminal_printf!(self, SET_COL_FORMAT.as_ptr(), col + 1);
    Ok(())
  }

  pub fn reset(&mut self) {
    unsafe {
      fclose(self.fout);
    }
    // it isn't the best if we can't reset the terminal but at least don't mess with
    // the output of the matches
    if tcsetattr(self.fdin, TCSANOW, &self.original_termios).is_ok() {
      unsafe {
        close(self.fdin);
      }
    }
  }

  pub fn get_reader(&self) -> TtyReader {
    TtyReader { fdin: self.fdin }
  }
}

pub(crate) struct TtyReader {
  fdin: i32,
}

impl TtyReader {
  pub fn read(&self) -> Option<[u8; 5]> {
    let mut input: [u8; 5] = [0; 5];
    let n_read = unsafe { read(self.fdin, input.as_mut_ptr() as *mut c_void, 4) };

    // loop {
    //   unsafe {
    //     // let mut fdset: fd_set = std::mem::uninitialized::<libc::fd_set>();
    //     let mut fdset: fd_set = std::mem::MaybeUninit::zeroed().assume_init();
    //     FD_ZERO(&mut fdset);
    //     FD_SET(self.fdin, &mut fdset);
    //     let timeout = timespec {
    //       tv_sec: 0,
    //       tv_nsec: 0,
    //     };
    //   }
    //   break;
    // }

    if n_read <= 0 {
      // a signal interrupted
      None
    } else {
      Some(input)
    }
  }
}
