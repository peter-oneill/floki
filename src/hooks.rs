use crate::errors::FlokiError;
use anyhow::Error;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    process::{Command, Stdio},
};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct HookConfig {
    pub(crate) pre_render_hook: Option<String>,
}

pub fn find_pre_render_hook(file: &Path) {
    let content = std::fs::read_to_string(file)
        .map_err(|e| FlokiError::ProblemOpeningConfigYaml {
            name: file.display().to_string(),
            error: e,
        })
        .unwrap();

    let matcher = Regex::new("pre_render_hook: *(?<numbers>.*)").unwrap();

    let myfoo = matcher
        .captures(&content)
        .map(|c| c.name("numbers").unwrap().as_str().to_string());

    run_command(&myfoo, file);
}

pub fn run_command(cmd: &Option<String>, config_path: &Path) -> Result<(), Error> {
    if let None = &cmd {
        return Ok(());
    }

    let cmd = cmd.clone().unwrap();

    let working_dir = config_path
        .parent()
        .expect("File should have a parent directory");

    debug!("Running command: {:?}", cmd);

    let mut cmd_elements = cmd.split_whitespace();
    let exe = cmd_elements.next().unwrap();
    let args: Vec<&str> = cmd_elements.collect();

    let status = Command::new(exe)
        .args(args)
        .current_dir(working_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| FlokiError::FailedToRunHook {
            hook: cmd.clone(),
            error: e,
        })?
        .status;

    if status.success() {
        Ok(())
    } else {
        Err(FlokiError::HookFailed {
            hook: cmd,
            status: status,
        }
        .into())
    }
}
