use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use serde::{Deserialize};
use serde_yaml;
use serde_inline_default::serde_inline_default;

pub type AppConfigs = BTreeMap<String, AppConfig>;

#[derive(Deserialize, PartialEq, Debug, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub ignore: bool,
    
    #[serde(default)]
    pub ignore_arguments: bool,
    
    #[serde(default)]
    pub single_instance: bool,

    #[serde(default)]
    pub extra_args: String,

    #[serde(default)]
    pub apply_windowrules: bool
}

#[serde_inline_default]
#[derive(Deserialize, PartialEq, Debug)]
pub struct Config {
    #[serde_inline_default(60)]
    pub save_interval: u64,

    #[serde_inline_default(std::env::var("HOME").unwrap() + "/.local/share/hyprsession")]
    pub session_path: String,

    #[serde_inline_default(BTreeMap::new())]
    pub apps: AppConfigs 
}

pub fn load_config(path: &String) -> Config {
    log::info!("Looking for config file at: {}", path);
    let result = File::open(path);

    if result.is_ok() {
        log::info!("Found config file");
        let reader = BufReader::new(result.unwrap());
        return serde_yaml::from_reader::<BufReader<File>, Config>(reader).expect("Failed to parse config file");
    } 

    return serde_yaml::from_str("").unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_test() {
        let path = String::from("data/config.yaml");
        let conf = load_config(&path);
        assert_eq!(conf.save_interval, 100);
        assert_eq!(conf.session_path, "~/.local/share/hypr");

        let apps = conf.apps; 
        let discord = apps.get("discord").unwrap();
        assert_eq!(discord.ignore, false); 
        assert_eq!(discord.ignore_arguments, true);
        assert_eq!(discord.single_instance, true);
    }

    #[test]
    fn no_config_test() {
        let path = String::from("data/noconfig.yaml"); // File doesn't exist
        let conf = load_config(&path);
        assert_eq!(conf.save_interval, 60);
        assert_eq!(conf.session_path, std::env::var("HOME").unwrap() + "/.local/share/hyprsession");
        assert_eq!(conf.apps, BTreeMap::new());
    }
}