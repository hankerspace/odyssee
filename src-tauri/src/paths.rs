//! Runtime path resolution and storage bootstrap helpers.

use crate::constants::{DB_FILE_NAME, IMAGE_MODEL_FILE, LLM_MODEL_FILE};
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Clone)]
pub(crate) struct RuntimePaths {
  pub(crate) data_dir: PathBuf,
  pub(crate) model_dir: PathBuf,
  pub(crate) cache_dir: PathBuf,
  pub(crate) image_cache_dir: PathBuf,
  pub(crate) database_path: PathBuf,
  pub(crate) llm_binary: PathBuf,
  pub(crate) image_binary: PathBuf,
  pub(crate) llm_model: PathBuf,
  pub(crate) image_model: PathBuf,
}

impl RuntimePaths {
  /// Resolves all local paths without creating folders on disk.
  pub(crate) fn resolve() -> Self {
    let data_dir = resolve_data_directory();
    let model_dir = data_dir.join("models");
    let cache_dir = data_dir.join("cache");
    let bin_dir = data_dir.join("bin");

    let paths = Self {
      data_dir: data_dir.clone(),
      model_dir: model_dir.clone(),
      cache_dir: cache_dir.clone(),
      image_cache_dir: cache_dir.join("images"),
      database_path: data_dir.join(DB_FILE_NAME),
      llm_binary: bin_dir.join(executable_name("llama-cli")),
      image_binary: bin_dir.join(executable_name("sd")),
      llm_model: model_dir.join(LLM_MODEL_FILE),
      image_model: model_dir.join(IMAGE_MODEL_FILE),
    };
    log::info!("Resolved runtime paths under '{}'", data_dir.display());
    paths
  }
}

/// Creates the local storage directories required by the backend.
pub(crate) fn ensure_storage(paths: &RuntimePaths) -> Result<(), String> {
  log::info!("Ensuring local storage directories under '{}'", paths.data_dir.display());
  fs::create_dir_all(&paths.data_dir).map_err(|error| error.to_string())?;
  fs::create_dir_all(&paths.model_dir).map_err(|error| error.to_string())?;
  fs::create_dir_all(&paths.cache_dir).map_err(|error| error.to_string())?;
  fs::create_dir_all(&paths.image_cache_dir).map_err(|error| error.to_string())?;
  Ok(())
}

pub(crate) fn resolve_model_directory() -> String {
  RuntimePaths::resolve().model_dir.display().to_string()
}

pub(crate) fn resolve_cache_directory() -> String {
  RuntimePaths::resolve().cache_dir.display().to_string()
}

fn executable_name(name: &str) -> String {
  if cfg!(target_os = "windows") {
    format!("{name}.exe")
  } else {
    name.to_string()
  }
}

fn resolve_data_directory() -> PathBuf {
  if cfg!(target_os = "windows") {
    if let Ok(app_data) = env::var("APPDATA") {
      // Match the Windows runtime layout documented for the MVP.
      return PathBuf::from(app_data).join("Odyssee");
    }
  }

  env::var("HOME")
    .map(|home| PathBuf::from(home).join(".local/share/Odyssee"))
    .unwrap_or_else(|_| PathBuf::from("Odyssee"))
}