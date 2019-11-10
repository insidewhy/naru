extern crate termios;
use std::ffi::CString;
use std::io;
use std::io::{Error, ErrorKind};
use self::termios::{Termios, ICRNL, ICANON, ECHO, ISIG, TCSANOW, tcsetattr};
use libc::{setvbuf, _IOFBF};

pub struct Tty {
  fdin: i32,
  fout: * mut libc::FILE,
  original_termios: Termios,
  fg_color: i32,
  max_width: i32,
  max_height: i32,
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
      return Err(Error::new(ErrorKind::Other, "Could not setvbuf"));
    }

    Ok(
      Tty {
        fdin,
        fout,
        fg_color: 9,
        original_termios,
        // TODO: query for these
        max_width: 80,
        max_height: 25,
      }
    )
  }

  pub fn reset(&self) -> () {
    tcsetattr(self.fdin, TCSANOW, &self.original_termios);
  }
}
