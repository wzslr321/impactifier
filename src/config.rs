use serde::{Deserialize, Deserializer};
use std::cmp;
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
    pub url: Option<Url>,
    pub path: Option<Box<Path>>,
    pub access_token: Option<String>,
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
    pub clone_into: Option<Box<Path>>,
}

impl Config {
    pub fn load_from_file(file_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
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
