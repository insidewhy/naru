#[macro_export]
macro_rules! def_c_str {
  ($($name: ident = $string: expr);+;) => {
    $(
      const $name: &'static [u8] = $string;
    )+
  };
}

#[macro_export]
macro_rules! c_str {
  ($str: ident) => {
    $str.as_ptr() as *const i8;
  };
}
