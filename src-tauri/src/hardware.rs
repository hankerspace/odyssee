//! Hardware detection and accelerator flag selection.

use crate::constants::{SAFETY_PROMPT, STYLE_WRAPPER};
use crate::i18n::backend_messages;
use crate::models::{HardwareProfile, RuntimeProfile};
use crate::paths::{resolve_cache_directory, resolve_model_directory, RuntimePaths};
use std::env;
use std::process::Command;

/// Produces a best-effort local inference profile without contacting external services.
pub(crate) fn detect_hardware_profile() -> HardwareProfile {
  if cfg!(target_os = "macos") {
    log::info!("Hardware detection selected Metal profile for macOS");
    return HardwareProfile {
      hardware: "Mac (Apple Silicon)".to_string(),
      accelerator: "Metal".to_string(),
      expected_performance: "Very smooth".to_string(),
    };
  }

  if has_command("nvidia-smi") {
    log::info!("Hardware detection selected CUDA profile from nvidia-smi");
    return HardwareProfile {
      hardware: "NVIDIA RTX".to_string(),
      accelerator: "CUDA".to_string(),
      expected_performance: "Near-instant inference".to_string(),
    };
  }

  if env::var("VULKAN_SDK").is_ok() || env::var("VK_ICD_FILENAMES").is_ok() {
    log::info!("Hardware detection selected Vulkan profile from environment variables");
    return HardwareProfile {
      hardware: "AMD / Intel GPU".to_string(),
      accelerator: "Vulkan".to_string(),
      expected_performance: "Good fluidity".to_string(),
    };
  }

  log::info!("Hardware detection selected CPU profile");
  HardwareProfile {
    hardware: "PC without dedicated GPU".to_string(),
    accelerator: "CPU (AVX2/AVX512 when available)".to_string(),
    expected_performance: "Text is smooth, images in 15-45 seconds".to_string(),
  }
}

/// Builds the complete runtime profile sent to the frontend status panel.
pub(crate) fn build_runtime_profile(locale: &str) -> RuntimeProfile {
  let hardware_profile = localized_hardware_profile(locale);
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

fn localized_hardware_profile(locale: &str) -> HardwareProfile {
  let mut hardware_profile = detect_hardware_profile();
  let messages = backend_messages(locale).hardware;

  match hardware_profile.accelerator.as_str() {
    "Metal" => hardware_profile.expected_performance = messages.very_smooth,
    "CUDA" => hardware_profile.expected_performance = messages.near_instant,
    "Vulkan" => hardware_profile.expected_performance = messages.good_fluidity,
    _ => {
      hardware_profile.hardware = messages.pc_without_gpu;
      hardware_profile.expected_performance = messages.cpu_performance;
    }
  }

  hardware_profile
}

fn accelerator_flags(accelerator: &str) -> (Vec<String>, Vec<String>) {
  // Keep sidecar flags centralized so command handlers only orchestrate execution.
  match accelerator {
    "CUDA" => (
      vec!["--ctx-size".into(), "4096".into(), "--n-gpu-layers".into(), "99".into()],
      vec!["--steps".into(), "4".into()],
    ),
    "Metal" => (
      vec!["--ctx-size".into(), "4096".into(), "--n-gpu-layers".into(), "99".into()],
      vec!["--steps".into(), "4".into()],
    ),
    "Vulkan" => (
      vec!["--ctx-size".into(), "4096".into(), "--backend".into(), "vulkan".into()],
      vec!["--steps".into(), "4".into()],
    ),
    _ => (
      vec!["--ctx-size".into(), "4096".into(), "--threads".into(), "4".into()],
      vec!["--steps".into(), "4".into()],
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn metal_llm_flags_do_not_use_removed_metal_argument() {
    let (llm_flags, _) = accelerator_flags("Metal");

    assert!(!llm_flags.contains(&"--metal".to_string()));
  }

  #[test]
  fn image_flags_do_not_pass_unsupported_backend_argument() {
    for accelerator in ["CUDA", "Metal", "Vulkan", "CPU (AVX2/AVX512 when available)"] {
      let (_, image_flags) = accelerator_flags(accelerator);

      assert!(!image_flags.contains(&"--backend".to_string()));
    }
  }
}