mod config;
mod envexpand;

use std::path::{Path, PathBuf};
use std::process::{exit, Command};
use std::env;

fn git_toplevel(cwd: &Path) -> Option<PathBuf> {
    let out = Command::new("git")
        .arg("rev-parse")
        .arg("--show-toplevel")
        .current_dir(cwd)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }

    let s = String::from_utf8(out.stdout).ok()?;
    let path = s.trim();
    Some(PathBuf::from(path))
}

fn is_inside_suna() -> bool {
    match std::env::var("SUNA") {
        Ok(v) => v == "1",
        Err(_) => false,
    }
}



fn run_hook_script(script: &PathBuf, profile_name: &str, phase: &str, exit_code: Option<i32>) {
    let mut cmd = Command::new("sh");
    cmd.arg(script);

    cmd.env("SUNA_PROFILE", profile_name);
    cmd.env("SUNA_PHASE", phase);
    if let Some(code) = exit_code {
        cmd.env("SUNA_EXIT_STATUS", code.to_string());
    }

    let status = cmd.status().unwrap_or_else(|e| {
        panic!(
            "failed to run {phase} script {}: {e}",
            script.display()
        );
    });

    if !status.success() {
        eprintln!(
            "warning: {phase} script {} exited with status {:?}",
            script.display(),
            status.code()
        );
    }
}

fn main() {

    let cwd = env::current_dir().unwrap_or_else(|e| {
        panic!("failed to get current dir: {e}");
    });
    if is_inside_suna() {
        return;
    }
    if git_toplevel(&cwd).is_none() {
    // git管理下ではない場合
        return;
    }

    let cfg = config::load_default_config();

    let profile_name = &cfg.app.default_profile;
    let profile = cfg.profiles.get(profile_name).unwrap_or_else(|| {
        panic!("profile not found: {profile_name}");
    });

    let hooks = cfg.hooks.get(profile_name);

    if let Some(hook) = cfg.hooks.get(profile_name) {
        if let Some(script) = &hook.pre_script {
            Command::new("sh").arg(script).status().unwrap_or_else(|e| {
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
    
    cmd.env("SUNA", "1");
    cmd.env("SUNA_PROFILE", profile_name);
    let shell = std::env::var("$SHELL").unwrap_or_else(|_| "zsh".into());

    cmd.arg("--");
    cmd.arg(shell);
    for (k, v) in &profile.env {
        cmd.env(k, v);
    }
    let status = cmd.status().unwrap_or_else(|e| {
        panic!("failed to execute sandbox-exec: {e}");
    });

    let code = status.code().unwrap_or(1);

    if let Some(h) = hooks {
        if let Some(script) = &h.post_script {
            run_hook_script(script, profile_name, "post", Some(code));
        }
    }

    exit(code);
}
