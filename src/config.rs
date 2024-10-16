use anyhow::Result;
use serde::{Deserialize, Deserializer};
use std::cmp;
use std::fmt;
use std::path::Path;
use thiserror::Error;
use tracing::debug;
use tracing::error;
use url::Url;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config from path: {}. Error:{}", path, msg)]
    ReadFailure { path: String, msg: String },
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub repository: RepositoryConfig,
    pub options: OptionsConfig,
    pub rules: Vec<Rule>,
}

#[derive(Debug, Deserialize)]
pub struct RepositoryConfig {
    #[serde(deserialize_with = "deserialize_url", default)]
    pub url: Option<Url>,
    pub path: Option<Box<Path>>,
    pub access_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OptionsConfig {
    pub clone_into: Option<Box<Path>>,
}

#[derive(Debug, Deserialize)]
pub struct TransformStep {
    pub name: String,
    #[serde(default)]
    pub args: Option<serde_yaml::Value>,
}

#[derive(Debug, Deserialize)]
pub struct Transform {
    // pub name: String,
    #[serde(default)]
    pub steps: Vec<TransformStep>,
}

#[derive(Debug, Deserialize)]
pub enum AlertLevel {
    Info,
    Warn,
    Severe,
}

#[derive(Debug, Deserialize)]
pub struct Action {
    // pub alert_level: AlertLevel,
    // pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct Rule {
    pub transform: Transform,
}

pub struct CustomStep {
    pub name: String,
    pub script: String,
}

impl Config {
    // TODO(wiktor.zajac) improve error handling
    pub fn load_from_file(file_path: &Path) -> Result<Self> {
        let yaml_content = match std::fs::read_to_string(file_path) {
            Ok(content) => {
                debug!("Succesfully read yaml config file:\n{}", content);
                content
            }
            Err(e) => {
                error!("Failed to read yaml config");
                return Err(ConfigError::ReadFailure {
                    path: String::from(file_path.to_string_lossy()),
                    msg: e.to_string(),
                }
                .into());
            }
        };

        let cfg = serde_yaml::from_str(&yaml_content)?;
        debug!("Deserialized config:\n{}", cfg);

        Ok(cfg)
    }

    pub fn custom_transform_scripts(&self) -> Option<Vec<CustomStep>> {
        let scripts: Vec<CustomStep> = self
            .rules
            .iter()
            .flat_map(|rule| &rule.transform.steps) 
            .filter(|step| step.name.starts_with("custom")) 
            .filter_map(|step| {
                step.args
                    .as_ref()
                    .and_then(|args| args.get("script"))
                    .and_then(|script_value| script_value.as_str())
                    .map(|s| CustomStep {
                        name: step.name.to_owned(),
                        script: s.to_string(),
                    })
            })
            .collect();

        match &scripts.is_empty() {
            true => None,
            false => Some(scripts),
        }
    }
}

impl fmt::Display for RepositoryConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RepositoryConfig {{ url: {},  access_token: {} }}",
            match &self.url {
                Some(url) => url.as_str(),
                None => "None",
            },
            match &self.access_token {
                Some(token) => {
                    let last_characters =
                        match token.char_indices().nth_back(cmp::min(4, token.len())) {
                            Some((i, _)) => &token[i..],
                            // TODO: Improve somehow
                            None => "INVALID",
                        };

                    format!("****{}", last_characters)
                }
                None => "None".to_string(),
            }
        )
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Config {{ repository: {}, options: {:?}, rules: {:?} }}",
            self.repository, self.options, self.rules,
        )
    }
}

fn deserialize_url<'a, D>(deserializer: D) -> Result<Option<Url>, D::Error>
where
    D: Deserializer<'a>,
{
    let url_str = String::deserialize(deserializer)?;
    match Url::parse(&url_str) {
        Ok(url) => Ok(Some(url)),
        Err(e) => Err(serde::de::Error::custom(e)),
    }
}
