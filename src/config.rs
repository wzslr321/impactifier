use serde::{Deserialize, Deserializer};
use std::env;
use std::fmt;
use std::path::Path;
use tracing::debug;
use url::Url;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub repository: RepositoryConfig,
    pub options: OptionsConfig,
}

#[derive(Debug, Deserialize)]
pub struct RepositoryConfig {
    #[serde(deserialize_with = "deserialize_url")]
    pub url: Url,
    pub access_token: Option<String>,
    pub branch: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerAction {
    Push,
    PullRequest,
}

#[derive(Debug, Deserialize)]
pub struct OptionsConfig {
    pub on: Vec<TriggerAction>,
    pub clone_into: Box<Path>,
}

impl Config {
    pub fn load_from_file(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let yaml_content = match std::fs::read_to_string(file_path) {
            Ok(content) => {
                debug!("Succesfully read yaml config file:\n{}", content);
                content
            }
            Err(e) => {
                return Err(e.into());
            }
        };
        let yaml_content = replace_env_vars(&yaml_content);
        debug!("replaced env variables in yaml config");

        let cfg = serde_yaml::from_str(&yaml_content)?;
        debug!("Deserialized config:\n{}", cfg);

        Ok(cfg)
    }
}
// Implement Display for RepositoryConfig
impl fmt::Display for RepositoryConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RepositoryConfig {{ url: {}, branch: {}, access_token: {} }}",
            self.url,
            self.branch,
            if self.access_token.is_some() {
                "[CENSORED_ACCESS_TOKEN]"
            } else {
                "None"
            }
        )
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Config {{ repository: {}, options: {:?} }}",
            self.repository, self.options
        )
    }
}

// TODO: Probably could be globally handled with regex,
// but it first needs to be ensured that the performance
// will not be affected. If somehow the regex will affect
// performance strong enough, replacable variables should
// be predefined instead of being hardcoded here.
fn replace_env_vars(yaml_content: &str) -> String {
    yaml_content.replace(
        "${GITHUB_ACCESS_TOKEN}",
        &env::var("GITHUB_ACCESS_TOKEN").unwrap_or_default(),
    )
}

fn deserialize_url<'a, D>(deserializer: D) -> Result<Url, D::Error>
where
    D: Deserializer<'a>,
{
    let url_str = String::deserialize(deserializer)?;
    Url::parse(&url_str).map_err(serde::de::Error::custom)
}
