//! Sidecar process adapters for llama.cpp and stable-diffusion.cpp.

use crate::paths::RuntimePaths;
use std::path::Path;
use std::process::{Command, Stdio};

pub(crate) fn run_llama_sidecar(paths: &RuntimePaths, flags: &[String], prompt: &str) -> Result<String, String> {
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
    return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
  }

  Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub(crate) fn run_stable_diffusion_sidecar(paths: &RuntimePaths, flags: &[String], prompt: &str, output_path: &Path) -> Result<(), String> {
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
    Ok(())
  } else {
    Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
  }
}