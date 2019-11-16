use std::os::raw::c_char;

pub(crate) struct ConstCStr {
  pub val: &'static str,
}

impl ConstCStr {
  #[inline]
  pub fn as_ptr(&self) -> *const c_char {
    self.val.as_ptr() as *const c_char
  }
}

#[macro_export]
macro_rules! def_c_str {
  ($($name: ident = $string: expr);+;) => {
    $(
      const $name: c_str::ConstCStr = c_str::ConstCStr { val: concat!($string, "\0") };
    )+
  };
}
