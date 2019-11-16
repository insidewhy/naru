use libc::{
  close, fclose, fflush, fileno, fprintf, fputc, ioctl, setvbuf, winsize, TIOCGWINSZ, _IOFBF,
};
use std::{
  ffi::CString,
  io,
  io::{Error, ErrorKind},
};
use termios::{tcsetattr, Termios, ECHO, ICANON, ICRNL, ISIG, TCSANOW};

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

#[macro_use]
#[path = "c_str.rs"]
mod c_str;

def_c_str! {
  WRITE_FORMAT = b"w\0";
  CLEAR_LINE_FORMAT = b"\x1b[K\0";
  SGR_FORMAT = b"\x1b[%im\0";
  NEWLINE_FORMAT = b"\x1b[K\n\0";
  MOVE_UP_FORMAT = b"\x1b[%iA\0";
  SET_COL_FORMAT = b"\x1b[%iG\0";
  NO_WRAP_FORMAT = b"\x1b[?7l\0";
  WRAP_FORMAT = b"\x1b[?7h\0";
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
    let fout = unsafe { libc::fopen(tty_filename_c.as_ptr(), c_str!(WRITE_FORMAT)) };

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
    terminal_printf!(self, c_str!(SGR_FORMAT), code);
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
    terminal_printf!(self, c_str!(NO_WRAP_FORMAT));
    Ok(())
  }

  pub fn set_wrap(&self) -> io::Result<()> {
    terminal_printf!(self, c_str!(WRAP_FORMAT));
    Ok(())
  }

  // pub fn set_underline(&self) -> io::Result<()> { self.sgr(4) }

  pub fn set_normal(&mut self) -> io::Result<()> {
    self.sgr(0)?;
    self.fg_color = 9;
    Ok(())
  }

  pub fn move_up(&self, row_count: i32) -> io::Result<()> {
    terminal_printf!(self, c_str!(MOVE_UP_FORMAT), row_count);
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
    terminal_printf!(self, c_str!(CLEAR_LINE_FORMAT));
    Ok(())
  }

  // Remove everything after cursor then move to next line
  pub fn newline(&self) -> io::Result<()> {
    terminal_printf!(self, c_str!(NEWLINE_FORMAT));
    Ok(())
  }

  pub fn flush(&self) {
    unsafe { fflush(self.fout) };
  }

  pub fn set_col(&self, col: i32) -> io::Result<()> {
    terminal_printf!(self, c_str!(SET_COL_FORMAT), col + 1);
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
}
