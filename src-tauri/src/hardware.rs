//! Hardware detection and accelerator flag selection.

use crate::constants::{SAFETY_PROMPT, STYLE_WRAPPER};
use crate::models::{HardwareProfile, RuntimeProfile};
use crate::paths::{resolve_cache_directory, resolve_model_directory, RuntimePaths};
use std::env;
use std::process::Command;

/// Produces a best-effort local inference profile without contacting external services.
pub(crate) fn detect_hardware_profile() -> HardwareProfile {
  if cfg!(target_os = "macos") {
    return HardwareProfile {
      hardware: "Mac (Apple Silicon)".to_string(),
      accelerator: "Metal".to_string(),
      expected_performance: "Very smooth".to_string(),
    };
  }

  if has_command("nvidia-smi") {
    return HardwareProfile {
      hardware: "NVIDIA RTX".to_string(),
      accelerator: "CUDA".to_string(),
      expected_performance: "Near-instant inference".to_string(),
    };
  }

  if env::var("VULKAN_SDK").is_ok() || env::var("VK_ICD_FILENAMES").is_ok() {
    return HardwareProfile {
      hardware: "AMD / Intel GPU".to_string(),
      accelerator: "Vulkan".to_string(),
      expected_performance: "Good fluidity".to_string(),
    };
  }

  HardwareProfile {
    hardware: "PC without dedicated GPU".to_string(),
    accelerator: "CPU (AVX2/AVX512 when available)".to_string(),
    expected_performance: "Text is smooth, images in 15-45 seconds".to_string(),
  }
}

/// Builds the complete runtime profile sent to the frontend status panel.
pub(crate) fn build_runtime_profile() -> RuntimeProfile {
  let hardware_profile = detect_hardware_profile();
  let paths = RuntimePaths::resolve();
  let (llm_flags, image_flags) = accelerator_flags(&hardware_profile.accelerator);

  RuntimeProfile {
    hardware: hardware_profile.hardware,
    accelerator: hardware_profile.accelerator,
    expected_performance: hardware_profile.expected_performance,
    llm_flags,
    image_flags,
    model_directory: resolve_model_directory(),
    cache_directory: resolve_cache_directory(),
    safety_prompt: SAFETY_PROMPT.to_string(),
    style_wrapper: STYLE_WRAPPER.to_string(),
    llm_binary: paths.llm_binary.display().to_string(),
    image_binary: paths.image_binary.display().to_string(),
    llm_model: paths.llm_model.display().to_string(),
    image_model: paths.image_model.display().to_string(),
    database_path: paths.database_path.display().to_string(),
  }
}

fn accelerator_flags(accelerator: &str) -> (Vec<String>, Vec<String>) {
  match accelerator {
    "CUDA" => (
      vec!["--ctx-size".into(), "4096".into(), "--n-gpu-layers".into(), "99".into()],
      vec!["--steps".into(), "4".into(), "--backend".into(), "cuda".into()],
    ),
    "Metal" => (
      vec!["--ctx-size".into(), "4096".into(), "--metal".into()],
      vec!["--steps".into(), "4".into(), "--backend".into(), "metal".into()],
    ),
    "Vulkan" => (
      vec!["--ctx-size".into(), "4096".into(), "--backend".into(), "vulkan".into()],
      vec!["--steps".into(), "4".into(), "--backend".into(), "vulkan".into()],
    ),
    _ => (
      vec!["--ctx-size".into(), "4096".into(), "--threads".into(), "4".into()],
      vec!["--steps".into(), "4".into(), "--backend".into(), "cpu".into()],
    ),
  }
}

fn has_command(command: &str) -> bool {
  Command::new(command)
    .arg("--help")
    .output()
    .map(|result| result.status.success())
    .unwrap_or(false)
}