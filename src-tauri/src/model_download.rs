//! First-start model bootstrap helpers.

use crate::constants::{IMAGE_MODEL_FILE, IMAGE_MODEL_URL, LLM_MODEL_FILE, LLM_MODEL_URL};
use crate::models::{ModelAssetStatus, ModelPreparationStatus, ModelStatus};
use crate::paths::{ensure_storage, RuntimePaths};
use std::fs::{self, File};
use std::process::{Command, Stdio};
use std::path::{Path, PathBuf};

struct ModelAsset {
  label: &'static str,
  file_name: &'static str,
  url: &'static str,
  path: PathBuf,
}

/// Builds the model and sidecar readiness payload without starting downloads.
pub(crate) fn get_model_status_for_paths(paths: &RuntimePaths) -> ModelStatus {
  let llm_model_ready = paths.llm_model.exists();
  let image_model_ready = paths.image_model.exists();
  let llm_ready = paths.llm_binary.exists() && llm_model_ready;
  let image_ready = paths.image_binary.exists() && image_model_ready;

  ModelStatus {
    model_directory: paths.model_dir.display().to_string(),
    cache_directory: paths.cache_dir.display().to_string(),
    llm_binary: paths.llm_binary.display().to_string(),
    image_binary: paths.image_binary.display().to_string(),
    llm_model: paths.llm_model.display().to_string(),
    image_model: paths.image_model.display().to_string(),
    llm_model_ready,
    image_model_ready,
    llm_ready,
    image_ready,
    database_path: paths.database_path.display().to_string(),
    downloads: model_preparation_status(paths),
  }
}

pub(crate) fn prepare_models_for_paths(paths: &RuntimePaths) -> Result<ModelPreparationStatus, String> {
  ensure_storage(paths)?;
  log::info!("Preparing optional local model assets in '{}'", paths.model_dir.display());

  let llm = prepare_asset(&llm_asset(paths));
  let image = prepare_asset(&image_asset(paths));

  Ok(ModelPreparationStatus { llm, image })
}

pub(crate) fn model_preparation_status(paths: &RuntimePaths) -> ModelPreparationStatus {
  log::info!("Checking optional local model preparation status");
  ModelPreparationStatus {
    llm: status_for_asset(&llm_asset(paths), false, None),
    image: status_for_asset(&image_asset(paths), false, None),
  }
}

fn llm_asset(paths: &RuntimePaths) -> ModelAsset {
  ModelAsset {
    label: "LLM Phi-4 Mini",
    file_name: LLM_MODEL_FILE,
    url: LLM_MODEL_URL,
    path: paths.llm_model.clone(),
  }
}

fn image_asset(paths: &RuntimePaths) -> ModelAsset {
  ModelAsset {
    label: "Image FLUX.2 Klein",
    file_name: IMAGE_MODEL_FILE,
    url: IMAGE_MODEL_URL,
    path: paths.image_model.clone(),
  }
}

fn prepare_asset(asset: &ModelAsset) -> ModelAssetStatus {
  if asset.path.exists() {
    log::info!("Model asset already present: label='{}', path='{}'", asset.label, asset.path.display());
    return status_for_asset(asset, false, None);
  }

  log::info!("Model asset missing; attempting explicit download: label='{}'", asset.label);
  match download_asset(asset) {
    Ok(()) => {
      log::info!("Model asset downloaded: label='{}', path='{}'", asset.label, asset.path.display());
      status_for_asset(asset, true, None)
    }
    Err(error) => {
      log::warn!("Model asset download failed: label='{}', error='{error}'", asset.label);
      status_for_asset(asset, false, Some(error))
    }
  }
}

fn download_asset(asset: &ModelAsset) -> Result<(), String> {
  if let Some(parent) = asset.path.parent() {
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
  }

  let part_path = partial_path(&asset.path);
  // Write to a temporary .part file so failed downloads never masquerade as ready models.
  let output = File::create(&part_path).map_err(|error| error.to_string())?;
  drop(output);

  let status = Command::new("curl")
    .args(["--location", "--fail", "--silent", "--show-error", "--output"])
    .arg(&part_path)
    .arg(asset.url)
    .stdin(Stdio::null())
    .status()
    .map_err(|error| format!("curl indisponible pour télécharger le modèle: {error}"))?;

  if !status.success() {
    let _ = fs::remove_file(&part_path);
    return Err(format!("curl a échoué avec le statut {status}"));
  }

  fs::rename(&part_path, &asset.path).map_err(|error| error.to_string())?;
  Ok(())
}

fn status_for_asset(asset: &ModelAsset, downloaded: bool, error: Option<String>) -> ModelAssetStatus {
  ModelAssetStatus {
    label: asset.label.to_string(),
    file_name: asset.file_name.to_string(),
    path: asset.path.display().to_string(),
    url: asset.url.to_string(),
    ready: asset.path.exists(),
    downloaded,
    error,
  }
}

fn partial_path(path: &Path) -> PathBuf {
  let mut file_name = path
    .file_name()
    .map(|value| value.to_string_lossy().to_string())
    .unwrap_or_else(|| "model.gguf".to_string());
  file_name.push_str(".part");
  path.with_file_name(file_name)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn partial_path_keeps_original_extension_visible() {
    let path = PathBuf::from("models/phi-4-mini-instruct-q4_k_m.gguf");
    assert_eq!(partial_path(&path), PathBuf::from("models/phi-4-mini-instruct-q4_k_m.gguf.part"));
  }

  #[test]
  fn missing_asset_status_is_not_ready_without_network() {
    let path = PathBuf::from("target/odyssee-missing-model-status-test.gguf");
    let asset = ModelAsset {
      label: "Test model",
      file_name: "test.gguf",
      url: "https://example.invalid/test.gguf",
      path,
    };
    let status = status_for_asset(&asset, false, None);

    assert!(!status.ready);
    assert!(!status.downloaded);
    assert!(status.error.is_none());
  }
}