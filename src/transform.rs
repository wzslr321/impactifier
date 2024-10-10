use crate::config::CustomStep;
use anyhow::Result;
use rhai::{Dynamic, Engine, Map, Scope};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use tracing::trace;

#[derive(Debug, Clone)]
pub struct Context {
    pub matched_string: String,
    pub file_path: PathBuf,
    // TODO(wiktor.zajac) possible improvement with some custom
    // class type to support further context access on it
    pub class_name: Option<String>,
}

pub trait TransformFn {
    fn execute(
        &self,
        context: &Context,
        args: Option<&serde_yaml::Value>,
    ) -> Result<String, String>;
}

lazy_static::lazy_static! {
    static ref TRANSFORM_REGISTRY: Mutex<HashMap<String, Box<dyn TransformFn + Send + Sync>>> = Mutex::new(HashMap::new());
}

fn register_transform(name: &str, func: Box<dyn TransformFn + Send + Sync>) {
    TRANSFORM_REGISTRY
        .lock()
        .unwrap()
        .insert(name.to_string(), func);
}

pub fn init_registry(custom_steps: Option<Vec<CustomStep>>) {
    trace!("Starting to register transform scripts functions");

    register_transform("toUpperCase", Box::new(ToLowerCase));
    register_transform("replace", Box::new(Replace));
    trace!("Standard functions registered");

    if let Some(steps) = custom_steps {
        for step in steps {
            trace!("Initializing custom function {}", &step.name);
            register_transform(
                &step.name,
                Box::new(CustomFunction {
                    script: step.script.to_string(),
                }),
            );
        }
    }
}

pub struct ToLowerCase;

impl TransformFn for ToLowerCase {
    fn execute(
        &self,
        context: &Context,
        _args: Option<&serde_yaml::Value>,
    ) -> Result<String, String> {
        Ok(context.matched_string.to_lowercase())
    }
}

pub struct Replace;

impl TransformFn for Replace {
    fn execute(
        &self,
        context: &Context,
        args: Option<&serde_yaml::Value>,
    ) -> Result<String, String> {
        if let Some(args) = args {
            let pattern = args.get("pattern").and_then(|v| v.as_str()).unwrap_or("");
            let with = args.get("with").and_then(|v| v.as_str()).unwrap_or("");
            Ok(context.matched_string.replace(pattern, with))
        } else {
            Err("Replace requires 'pattern' and 'with' arguments".to_string())
        }
    }
}

pub struct CustomFunction {
    pub script: String,
}

impl TransformFn for CustomFunction {
    fn execute(
        &self,
        context: &Context,
        _args: Option<&serde_yaml::Value>,
    ) -> Result<String, String> {
        let engine = Engine::new();

        let mut context_map = Map::new();
        context_map.insert(
            "matched_string".into(),
            context.matched_string.clone().into(),
        );
        context_map.insert(
            "file_path".into(),
            String::from(context.file_path.to_string_lossy()).into(),
        );
        if let Some(class_name) = &context.class_name {
            context_map.insert("class_name".into(), class_name.clone().into());
        }

        let mut scope = Scope::new();
        scope.push("context", context_map);

        let result: Dynamic = engine
            .eval_with_scope::<Dynamic>(&mut scope, &self.script)
            .map_err(|e| format!("Script error: {}", e))?;

        result
            .try_cast::<String>()
            .ok_or_else(|| "Script did not return a string".to_string())
    }
}
