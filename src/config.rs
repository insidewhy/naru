use serde::Deserialize;
use std::io;
use toml;
use xdg;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WindowConfig {
  pub height: u16,
}

impl Default for WindowConfig {
  fn default() -> Self {
    Self { height: 20 }
  }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Config {
  #[serde(default)]
  pub window: WindowConfig,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      window: Default::default(),
    }
  }
}

pub(crate) fn load_config() -> io::Result<Config> {
  let xdg_dirs = xdg::BaseDirectories::new()?;
  let cfg_file = xdg_dirs.find_config_file("naru.toml");

  match cfg_file {
    None => Ok(Default::default()),

    Some(v) => {
      let path = v.to_str().unwrap();
      let content = std::fs::read_to_string(path)?;
      Ok(toml::from_str(&content)?)
    }
  }
}
