//! Sidecar process adapters for llama.cpp and stable-diffusion.cpp.

use crate::paths::RuntimePaths;
use std::path::Path;
use std::process::{Command, Stdio};

pub(crate) fn run_llama_sidecar(paths: &RuntimePaths, flags: &[String], prompt: &str) -> Result<String, String> {
  log::info!("Starting llama.cpp sidecar: binary='{}', model='{}'", paths.llm_binary.display(), paths.llm_model.display());
  let output = Command::new(&paths.llm_binary)
    .arg("-m")
    .arg(&paths.llm_model)
    .arg("-p")
    .arg(prompt)
    .arg("-n")
    .arg("420")
    .args(flags)
    .stdin(Stdio::null())
    .output()
    .map_err(|error| error.to_string())?;

  if !output.status.success() {
    log::warn!("llama.cpp sidecar exited with status {}", output.status);
    return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
  }

  log::info!("llama.cpp sidecar completed successfully");
  Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub(crate) fn run_stable_diffusion_sidecar(paths: &RuntimePaths, flags: &[String], prompt: &str, output_path: &Path) -> Result<(), String> {
  log::info!(
    "Starting stable-diffusion.cpp sidecar: binary='{}', model='{}', output='{}'",
    paths.image_binary.display(),
    paths.image_model.display(),
    output_path.display()
  );
  let output = Command::new(&paths.image_binary)
    .arg("-m")
    .arg(&paths.image_model)
    .arg("-p")
    .arg(prompt)
    .arg("-o")
    .arg(output_path)
    .args(flags)
    .stdin(Stdio::null())
    .output()
    .map_err(|error| error.to_string())?;

  if output.status.success() && output_path.exists() {
    log::info!("stable-diffusion.cpp sidecar completed successfully");
    Ok(())
  } else {
    log::warn!("stable-diffusion.cpp sidecar failed with status {}", output.status);
    Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
  }
}