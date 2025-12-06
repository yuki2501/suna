// src/main.rs
mod config;
mod envexpand;

use std::path::PathBuf;
use std::process::{Command, exit};

fn main() {
    let cfg = config::load_default_config();

    let profile_name = &cfg.app.default_profile;
    let profile = cfg.profiles.get(profile_name).unwrap_or_else(|| {
        panic!("profile not found: {profile_name}");
    });

    if let Some(hook) = cfg.hooks.get(profile_name) {
        if let Some(script) = &hook.pre_script {
            Command::new("sh")
                .arg(script)
                .status()
                .unwrap_or_else(|e| {
                    panic!("failed to run pre_script {}: {e}", script.display());
                });
        }
    }

    let sandbox_exec = cfg
        .app
        .sandbox_exec
        .clone()
        .unwrap_or_else(|| PathBuf::from("sandbox-exec"));

    let profile_path = if let Some(base) = &cfg.app.profiles_dir {
        if profile.profile_file.is_absolute() {
            profile.profile_file.clone()
        } else {
            base.join(&profile.profile_file)
        }
    } else {
        profile.profile_file.clone()
    };

    let mut cmd = Command::new(sandbox_exec);
    cmd.arg("-f").arg(&profile_path);

    for (k, v) in &profile.params {
        let expanded = envexpand::expand_env_vars(v);
        cmd.arg("-D");
        cmd.arg(format!("{k}={expanded}"));
    }

    cmd.arg("zsh"); // 本当はここに実行したいコマンドを載せる

    for (k, v) in &profile.env {
        cmd.env(k, v);
    }
    let status = cmd.status().unwrap_or_else(|e| {
        panic!("failed to execute sandbox-exec: {e}");
    });

    exit(status.code().unwrap_or(1));
}

