// src/config.rs
use serde::Deserialize;
use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub app: AppSection,
    #[serde(default)]
    pub profiles: HashMap<String, Profile>,
    #[serde(default)]
    pub hooks: HashMap<String, Hook>,
}

#[derive(Debug, Deserialize)]
pub struct AppSection {
    #[serde(default = "default_profile_name")]
    pub default_profile: String,
    /// sandbox-exec のパス。未指定なら "sandbox-exec" として PATH から探す。
    #[serde(default)]
    pub sandbox_exec: Option<PathBuf>,
    /// プロファイルファイルのベースディレクトリ
    #[serde(default)]
    pub profiles_dir: Option<PathBuf>,
}

fn default_profile_name() -> String {
    "default".to_string()
}

impl Default for AppSection {
    fn default() -> Self {
        AppSection {
            default_profile: default_profile_name(),
            sandbox_exec: None,
            profiles_dir: None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Profile {
    /// profiles_dir からの相対パス、または絶対パス
    pub profile_file: PathBuf,
    /// sandbox-exec に渡す -D KEY=VALUE を表すパラメータ
    #[serde(default)]
    pub params: HashMap<String, String>,
    /// ラップされるコマンドに渡す環境変数
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Hook {
    pub pre_script: Option<PathBuf>,
    pub post_script: Option<PathBuf>,
}

pub fn load_config(path: &Path) -> AppConfig {
    let text = fs::read_to_string(path).unwrap_or_else(|e| {
        panic!("failed to read config file {}: {e}", path.display());
    });

    toml::from_str(&text).unwrap_or_else(|e| {
        panic!("failed to parse config TOML {}: {e}", path.display());
    })
}

pub fn load_default_config() -> AppConfig {
    // XDG_CONFIG_HOME/suna/config.toml か ~/.config/suna/config.toml を見る
    let path = if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
        PathBuf::from(xdg).join("suna").join("config.toml")
    } else if let Ok(home) = env::var("HOME") {
        PathBuf::from(home)
            .join(".config")
            .join("suna")
            .join("config.toml")
    } else {
        panic!("HOME も XDG_CONFIG_HOME も見つからないので config パスを決められない");
    };

    load_config(&path)
}
