extern crate serde;
extern crate toml;
extern crate xdg;
use std::io;
use std::io::Read;
use std::fs::File;
use self::serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WindowConfig {
  pub height: u16,
}

impl Default for WindowConfig {
  fn default() -> Self {
    Self { height: 20 }
  }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
  #[serde(default)]
  pub window: WindowConfig,
}

impl Default for Config {
  fn default() -> Self {
    Self { window: Default::default() }
  }
}

pub fn load_config() -> io::Result<Config> {
  let xdg_dirs = xdg::BaseDirectories::new()?;
  let cfg_file = xdg_dirs.find_config_file("naru.toml");

  match cfg_file {
    None => {
      Ok(Default::default())
    }

    Some(v) => {
      let path = v.to_str().unwrap();
      let mut file = File::open(path)?;
      let mut content = String::new();
      file.read_to_string(&mut content)?;
      Ok(toml::from_str(&content)?)
    }
  }
}
