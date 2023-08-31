use crate::errors::FlokiError;
use anyhow::Error;
use fancy_regex::Regex;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    process::{Command, Stdio},
};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct HookConfig {
    pub(crate) pre_render_hook: Vec<String>,
}

lazy_static! {
    static ref PRE_RENDER_HOOK_REGEX: Regex =
        Regex::new(
            // (Multi-line match)
            // Starting with some amount of whitespace before a "pre_render_hook:" key
            // ...and any more characters on the same line
            // Then any number of lines that start with either
            // - the same amount of whitespace, followed by a dash
            //   ...and any more characters on the same line
            // - a greater amount of whitespace, followed by anything
            r"(?m)^(\s*)pre_render_hook:.*(\n\1[\s\-].+$)*").unwrap();
}

pub fn run_pre_render_hook(file: &Path) -> Result<(), FlokiError> {
    let content = std::fs::read_to_string(file)
        .map_err(|e| FlokiError::ProblemOpeningConfigYaml {
            name: file.display().to_string(),
            error: e,
        })
        .unwrap();

    let pre_render_hook = PRE_RENDER_HOOK_REGEX
        .find(&content)
        .expect("Invalid pre-render-hook regex")
        .map(|m| m.as_str())
        .unwrap_or_default();

    debug!("pre-render hook yaml:\n{}", pre_render_hook);

    let hook: HookConfig = serde_yaml::from_str(pre_render_hook).map_err(|e| {
        FlokiError::ProblemParsingConfigYamlSnippet {
            error: e,
            snippet: pre_render_hook.to_string(),
        }
    })?;

    run_hook(&hook.pre_render_hook, file)
}

pub fn run_hook(cmd: &Vec<String>, config_path: &Path) -> Result<(), FlokiError> {
    if cmd.is_empty() {
        return Ok(());
    }
    let (exe, args) = cmd.split_first().unwrap();

    let working_dir = config_path
        .parent()
        .expect("File should have a parent directory");

    debug!("Running command: {:?}", cmd);
    let cmd_str = cmd.join(" ");

    let status = Command::new(exe)
        .args(args)
        .current_dir(working_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| FlokiError::FailedToRunHook {
            hook: cmd_str.clone(),
            error: e,
        })?
        .status;

    if status.success() {
        Ok(())
    } else {
        Err(FlokiError::HookFailed {
            hook: cmd_str.clone(),
            status: status,
        })
    }
}
