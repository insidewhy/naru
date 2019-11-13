extern crate termios;
use std::ffi::CString;
use std::io;
use std::io::{Error, ErrorKind};
use self::termios::{Termios, ICRNL, ICANON, ECHO, ISIG, TCSANOW, tcsetattr};
use libc::{setvbuf, fprintf, fputc, _IOFBF, TIOCGWINSZ, ioctl, winsize, fileno, close, fclose};

// Make unsafe call and turn non-zero exit statuses into an io error with the given string when
// needed
macro_rules! fwd_error_code {
  ($expr: expr, $msg: expr) => {
    if unsafe { $expr != 0 } {
      return Err(Error::new(ErrorKind::Other, $msg))
    }
  }
}

macro_rules! c_str {
  ($expr: expr) => {
    CString::new($expr)?.as_ptr();
  }
}

pub struct Tty {
  fdin: i32,
  fout: * mut libc::FILE,
  original_termios: Termios,
  fg_color: i32,
  max_width: u16,
  max_height: u16,
}

impl Tty {
  pub fn new(tty_path: &str) -> io::Result<Tty> {
    let tty_filename_c = CString::new(tty_path)?;
    let fdin = unsafe { libc::open(tty_filename_c.as_ptr(), libc::O_RDONLY) };
    let fout = unsafe { libc::fopen(tty_filename_c.as_ptr(), c_str!("w")) };

    let original_termios = Termios::from_fd(fdin)?;

    let mut termios_copy = original_termios.clone();
    termios_copy.c_iflag &= !(ICRNL);
    termios_copy.c_lflag &= !(ICANON | ECHO | ISIG);
    tcsetattr(fdin, TCSANOW, &termios_copy)?;

    fwd_error_code!(
      setvbuf(fout, std::ptr::null_mut(), _IOFBF, 4096),
      "Could not setvbuf"
    );

    let mut ws = winsize { ws_row: 0, ws_col: 0, ws_xpixel: 0, ws_ypixel: 0 };
    fwd_error_code!(
      ioctl(fileno(fout), TIOCGWINSZ, &mut ws),
      "Could not get window size"
    );

    Ok(
      Tty {
        fdin,
        fout,
        fg_color: 0,
        original_termios,
        max_width: ws.ws_col,
        max_height: ws.ws_row,
      }
    )
  }

  pub fn sgr(&self, code: i32) -> io::Result<()>  {
    unsafe { fprintf(self.fout, c_str!("%c%c%im"), 0x1b, '[', code) };
    Ok(())
  }

  pub fn set_fg(&mut self, color: i32) -> io::Result<()> {
    if self.fg_color != color {
      self.sgr(30 + color)?;
      self.fg_color = color;
    }
    Ok(())
  }

  pub fn putc(&self, c: i32) {
    unsafe { fputc(c, self.fout) };
  }

  pub fn newline(&self) -> io::Result<()> {
    unsafe { fprintf(self.fout, c_str!("%c%cK\n"), 0x1b, '[') };
    Ok(())
  }

  pub fn reset(&mut self) {
    let _ = self.set_fg(0);
    unsafe { fclose(self.fout); }
    // it isn't the best if we can't reset the terminal but at least don't mess with
    // the output of the matches
    if tcsetattr(self.fdin, TCSANOW, &self.original_termios).is_ok() {
      unsafe { close(self.fdin); }
    }
  }
}
