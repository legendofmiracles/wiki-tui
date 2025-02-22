use dirs;
use ini::Ini;
use std::fs;
use reqwest;
use anyhow::*;

const CONFIG_FILE_NAME: &str = "config.ini";
const APP_CONFIG_DIR: &str = "wiki-tui";

pub struct ApiConfig {
    pub base_url: String
}

pub struct LoggingConfig {
    pub log_output: String,
    pub log_level: log::LevelFilter,
}

pub struct Config {
    pub api_config: Option<ApiConfig>,
    pub logging_config: Option<LoggingConfig>,
    pub config_path: Option<std::path::PathBuf>,
}

impl Config {
    pub fn new() -> Self {
        let mut config: Config = Default::default();
        
        let config_file_exists = config.config_file_exists();
        if !config_file_exists {
            config.create_config_file();
        }

        config.load();
        config
    }

    fn config_file_exists(&mut self) -> bool {
        match dirs::config_dir() {
            Some(config_path) => {
                let app_dir_path = config_path.join(APP_CONFIG_DIR);
                let config_file_path = app_dir_path.join(CONFIG_FILE_NAME);

                self.config_path = Some(config_file_path.clone());

                if !app_dir_path.exists() {
                    let result = fs::create_dir(app_dir_path).context("Failed to create the app config directory");
                    match result {
                        Ok(_) => log::info!("Created the app config directory"),
                        Err(error) => panic!("{:?}", error),
                    }
                    return false
                }

                if !config_file_path.exists() {
                    return false
                }

                true
            }

            None => {
                panic!("Couldn't find your config directory")
            }
        }
    }

    fn create_config_file(&mut self) {
        let file_url = "https://raw.githubusercontent.com/Builditluc/wiki-tui/stable/config.ini";
        let file_content = reqwest::blocking::get(file_url).unwrap().text().unwrap();

        let result = fs::write(&self.config_path.clone().unwrap(), file_content).context("Failed to create the config file");
        match result {
            Ok(_) => log::info!("Sucessfully created the config file"),
            Err(error) => panic!("{:?}", error),
        }
    }

    fn load_api(&mut self, config: &Ini) {
        // try to load the section
        let section = match config.section(Some("Api")) {
            Some(section) => section,
            None => return
        };

        // load every variable and if it doesn't exist,
        // use the default
        let base_url = match section.get("BASE_URL") {
            Some(base_url) => base_url.to_string(),
            None => "https://en.wikipedia.org/w/api.php".to_string()
        };

        self.api_config = Some(ApiConfig {
            base_url
        });
    }

    fn load_logging(&mut self, config: &Ini) {
        // try to load the section
        let section = match config.section(Some("Logging")) {
            Some(section) => section,
            None => return
        };
        
        // load every variable and if it doesn't exist,
        // use the default
        let log_output = match section.get("LOG_OUTPUT") {
            Some(log_output) => log_output.to_string(),
            None => "wiki_tui.log".to_string() 
        };

        let log_level = match section.get("LOG_LEVEL") {
            Some(log_level) => match log_level {
                "OFF" => log::LevelFilter::Off,
                "TRACE" => log::LevelFilter::Trace,
                "DEBUG" => log::LevelFilter::Debug,
                "INFO" => log::LevelFilter::Info,
                "WARN" => log::LevelFilter::Warn,
                "ERROR" => log::LevelFilter::Error,
                _ => log::LevelFilter::Off,
            },
            None => log::LevelFilter::Off
        };

        self.logging_config = Some(LoggingConfig {
            log_output,
            log_level
        }); 
    }

    fn load(&mut self) {
        let config = Ini::load_from_file(&self.config_path.clone().unwrap()).unwrap();
        self.load_logging(&config);
        self.load_api(&config);
    }

    pub fn get_logging_config(&self) -> &LoggingConfig {
        match self.logging_config {
            Some(ref logging_config) => logging_config,
            None => panic!("Holy Shit! What happened here!")
        }
    }

    pub fn get_api_config(&mut self) -> ApiConfig {
        let api_config = std::mem::replace(&mut self.api_config, None);
        if api_config.is_some() {
            return api_config.unwrap();
        } else {
            panic!("Holy Shit! What happened here!");
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            api_config: None,
            logging_config: None,
            config_path: None,
        }
    }
}
