use std::env;

use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub repository: RepositoryConfig,
    pub options: OptionsConfig,
}

#[derive(Debug, Deserialize)]
pub struct RepositoryConfig {
    pub path: String,
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
    pub branch: String,
    pub on: Vec<TriggerAction>,
}

impl Config {
    pub fn load_from_file(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let yaml_content = std::fs::read_to_string(file_path)?;
        let yaml_content = replace_env_vars(&yaml_content);

        let cfg = serde_yaml::from_str(&yaml_content)?;

        Ok(cfg)
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
