extern crate termios;
use std::ffi::CString;
use std::io;
use self::termios::Termios;
use libc;

pub struct Tty {
	fdin: i32,
	fout: * const libc::FILE,
	original_termios: Termios,
	fg_color: i32,
	max_width: i32,
	max_height: i32
}

impl Tty {
  pub fn init(tty_path: &str) -> io::Result<Tty> {

    let tty_filename_c = CString::new(tty_path)?;
    let fdin = unsafe { libc::open(tty_filename_c.as_ptr(), libc::O_RDONLY) };
    let original_termios = Termios::from_fd(fdin)?;

    let tty = Tty {
      fdin: fdin,
      fout: unsafe { libc::fopen(tty_filename_c.as_ptr(), CString::new("w")?.as_ptr()) },
      fg_color: 9,
      original_termios,
      // TODO: query for these
      max_width: 80,
      max_height: 25,
    };

    Ok(tty)
  }
}
