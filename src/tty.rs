extern crate termios;
use std::ffi::CString;
use std::io;
use std::io::{Error, ErrorKind};
use self::termios::{Termios, ICRNL, ICANON, ECHO, ISIG, TCSANOW, tcsetattr};
use libc::{setvbuf, _IOFBF, TIOCGWINSZ, ioctl, winsize, fileno, close, fclose};

macro_rules! io_err {
  ($x: expr) => {
    Err(Error::new(ErrorKind::Other, $x))
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
  pub fn init(tty_path: &str) -> io::Result<Tty> {
    let tty_filename_c = CString::new(tty_path)?;
    let fdin = unsafe { libc::open(tty_filename_c.as_ptr(), libc::O_RDONLY) };
    let fout = unsafe { libc::fopen(tty_filename_c.as_ptr(), CString::new("w")?.as_ptr()) };

    let original_termios = Termios::from_fd(fdin)?;

    let mut termios_copy = original_termios.clone();
    termios_copy.c_iflag &= !(ICRNL);
    termios_copy.c_lflag &= !(ICANON | ECHO | ISIG);
    tcsetattr(fdin, TCSANOW, &termios_copy)?;

    let err = unsafe { setvbuf(fout, std::ptr::null_mut(), _IOFBF, 4096) };
    if err != 0 {
      return io_err!("Could not setvbuf");
    }

    let mut ws = winsize { ws_row: 0, ws_col: 0, ws_xpixel: 0, ws_ypixel: 0 };
    let result = unsafe { ioctl(fileno(fout), TIOCGWINSZ, &mut ws) };
    if result != 0 {
      return io_err!("Could not get window size");
    }

    Ok(
      Tty {
        fdin,
        fout,
        fg_color: 9,
        original_termios,
        max_width: ws.ws_col,
        max_height: ws.ws_row,
      }
    )
  }

  pub fn reset(&self) -> io::Result<()> {
    tcsetattr(self.fdin, TCSANOW, &self.original_termios)?;
    unsafe {
      fclose(self.fout);
      close(self.fdin);
    };
    Ok(())
  }
}
