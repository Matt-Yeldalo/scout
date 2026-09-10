// use serde::Deserialize;
//
// #[derive(Deserialize)]
// #[serde(default)]
// pub struct Config {
//     pub debug: bool,
//     pub autosave: bool,
//     pub autosave_debounce_ms: u64,
//     pub data_dir: String,
//     pub config_path: String,
// }
//
// impl Config {
//     pub(crate) fn load() -> Result<Config, _> {
//     }
//
//     fn safe_get_config_path(self) -> PathBuf {
//         let path = PathBuf::from(self.config_path);
//     }
// }
//
// impl Default for Config {
//     fn default() -> Self {
//         Config {
//             debug: false,
//             autosave: true,
//             autosave_debounce_ms: 1000,
//             data_dir: String::from("./data"),
//             config_path: String::from("~/.config/scout/"),
//         }
//     }
// }

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Config {
    pub debug: bool,
    pub autosave: bool,
    pub autosave_debounce_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self { debug: false, autosave: true, autosave_debounce_ms: 800 }
    }
}
