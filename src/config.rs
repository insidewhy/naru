use crate::other_error;
use serde::Deserialize;
use std::{
  collections::HashMap,
  io,
  io::{Error, ErrorKind},
};
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

  #[serde(default)]
  pub bindings: HashMap<String, String>,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      window: Default::default(),
      bindings: HashMap::new(),
    }
  }
}

// Parse keys like "c-a" etc. into the corresponding control codes
fn parse_bindings(bindings: HashMap<String, String>) -> io::Result<HashMap<String, String>> {
  bindings
    .iter()
    .map(|(binding, action_name): (&String, &String)| {
      if binding.starts_with("c-") {
        if binding.len() != 3 {
          return other_error!(
            "Invalid binding, only one character allowed after c-: {}",
            binding
          );
        }
        let last_char = binding.bytes().last().unwrap();
        if last_char < b'a' || last_char > b'z' {
          return other_error!("Invalid binding, only a-z allowed after c-: {}", binding);
        }

        // I feel like there should be a better way to do this
        let control_code = String::from_utf8_lossy(&[last_char - b'`']).to_string();
        return Ok((control_code, action_name.clone()));
      } else {
        return other_error!("Invalid binding, must start with c-: {}", binding);
      }
    })
    .collect()
}

pub(crate) fn load_config() -> io::Result<Config> {
  let xdg_dirs = xdg::BaseDirectories::new()?;
  let cfg_file = xdg_dirs.find_config_file("naru.toml");

  match cfg_file {
    None => Ok(Default::default()),

    Some(v) => {
      let path = v.to_str().unwrap();
      let content = std::fs::read_to_string(path)?;
      let mut parsed_config: Config = toml::from_str(&content)?;
      parsed_config.bindings = parse_bindings(parsed_config.bindings)?;
      Ok(parsed_config)
    }
  }
}
