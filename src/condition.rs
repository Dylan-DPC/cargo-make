//! # condition
//!
//! Evaluates conditions based on task configuration and current env.
//!

#[cfg(test)]
#[path = "./condition_test.rs"]
mod condition_test;

use crate::command;
use crate::profile;
use crate::types;
use crate::types::{FlowInfo, RustVersionCondition, Step, TaskCondition};
use crate::version::is_newer;
use rust_info;
use rust_info::types::{RustChannel, RustInfo};
use std::env;

fn validate_env(condition: &TaskCondition) -> bool {
    let env = condition.env.clone();

    match env {
        Some(env_vars) => {
            let mut all_valid = true;

            for (key, current_value) in env_vars.iter() {
                match env::var(key) {
                    Ok(value) => {
                        all_valid = value == current_value.to_string();
                    }
                    _ => {
                        all_valid = false;
                    }
                };

                if !all_valid {
                    break;
                }
            }

            all_valid
        }
        None => true,
    }
}

fn validate_env_set(condition: &TaskCondition) -> bool {
    let env = condition.env_set.clone();

    match env {
        Some(env_vars) => {
            let mut all_valid = true;

            for key in env_vars.iter() {
                match env::var(key) {
                    Err(_) => {
                        all_valid = false;
                    }
                    _ => (),
                };

                if !all_valid {
                    break;
                }
            }

            all_valid
        }
        None => true,
    }
}

fn validate_env_not_set(condition: &TaskCondition) -> bool {
    let env = condition.env_not_set.clone();

    match env {
        Some(env_vars) => {
            let mut all_valid = true;

            for key in env_vars.iter() {
                match env::var(key) {
                    Ok(_) => {
                        all_valid = false;
                    }
                    _ => (),
                };

                if !all_valid {
                    break;
                }
            }

            all_valid
        }
        None => true,
    }
}

fn validate_platform(condition: &TaskCondition) -> bool {
    let platforms = condition.platforms.clone();
    match platforms {
        Some(platform_names) => {
            let platform_name = types::get_platform_name();

            let index = platform_names
                .iter()
                .position(|value| *value == platform_name);

            match index {
                None => {
                    debug!(
                        "Failed platform condition, current platform: {}",
                        &platform_name
                    );
                    false
                }
                _ => true,
            }
        }
        None => true,
    }
}

fn validate_profile(condition: &TaskCondition) -> bool {
    let profiles = condition.profiles.clone();
    match profiles {
        Some(profile_names) => {
            let profile_name = profile::get();

            let index = profile_names
                .iter()
                .position(|value| *value == profile_name);

            match index {
                None => {
                    debug!(
                        "Failed profile condition, current profile: {}",
                        &profile_name
                    );
                    false
                }
                _ => true,
            }
        }
        None => true,
    }
}

fn validate_channel(condition: &TaskCondition, flow_info: &FlowInfo) -> bool {
    let channels = condition.channels.clone();
    match channels {
        Some(channel_names) => match flow_info.env_info.rust_info.channel {
            Some(value) => {
                let index = match value {
                    RustChannel::Stable => channel_names
                        .iter()
                        .position(|value| *value == "stable".to_string()),
                    RustChannel::Beta => channel_names
                        .iter()
                        .position(|value| *value == "beta".to_string()),
                    RustChannel::Nightly => channel_names
                        .iter()
                        .position(|value| *value == "nightly".to_string()),
                };

                match index {
                    None => {
                        debug!("Failed channel condition");
                        false
                    }
                    _ => true,
                }
            }
            None => false,
        },
        None => true,
    }
}

fn validate_rust_version_condition(rustinfo: RustInfo, condition: RustVersionCondition) -> bool {
    if rustinfo.version.is_some() {
        let current_version = rustinfo.version.unwrap();

        let mut valid = match condition.min {
            Some(version) => {
                version == current_version || is_newer(&version, &current_version, true)
            }
            None => true,
        };

        if valid {
            valid = match condition.max {
                Some(version) => {
                    version == current_version || is_newer(&current_version, &version, true)
                }
                None => true,
            };
        }

        if valid {
            valid = match condition.equal {
                Some(version) => version == current_version,
                None => true,
            };
        }

        valid
    } else {
        true
    }
}

fn validate_rust_version(condition: &TaskCondition) -> bool {
    let rust_version = condition.rust_version.clone();
    match rust_version {
        Some(rust_version_condition) => {
            let rustinfo = rust_info::get();

            validate_rust_version_condition(rustinfo, rust_version_condition)
        }
        None => true,
    }
}

fn validate_criteria(flow_info: &FlowInfo, step: &Step) -> bool {
    match step.config.condition {
        Some(ref condition) => {
            debug!("Checking task condition structure.");

            validate_platform(&condition)
                && validate_profile(&condition)
                && validate_channel(&condition, &flow_info)
                && validate_env(&condition)
                && validate_env_set(&condition)
                && validate_env_not_set(&condition)
                && validate_rust_version(&condition)
        }
        None => true,
    }
}

fn validate_script(step: &Step) -> bool {
    match step.config.condition_script {
        Some(ref script) => {
            debug!("Checking task condition script.");

            let exit_code =
                command::run_script(&script, step.config.script_runner.clone(), &vec![], false);

            if exit_code == 0 {
                true
            } else {
                false
            }
        }
        None => true,
    }
}

pub(crate) fn validate_condition(flow_info: &FlowInfo, step: &Step) -> bool {
    validate_criteria(&flow_info, &step) && validate_script(&step)
}
