use serde::Serialize;
use std::env;
use std::path::PathBuf;
use std::process::Command;

const SAFETY_PROMPT: &str = "You are a children encyclopedia. Never mention violent, political, or inappropriate content. Stay factual and kind.";
const STYLE_WRAPPER: &str = "A professional educational illustration of [SUBJECT], sticker style, clean lines, bright colors, white background, high quality for children encyclopedia.";

#[derive(Serialize)]
struct RuntimeProfile {
  hardware: String,
  accelerator: String,
  expected_performance: String,
  llm_flags: Vec<String>,
  image_flags: Vec<String>,
  model_directory: String,
  cache_directory: String,
  safety_prompt: String,
  style_wrapper: String,
}

#[derive(Serialize)]
struct HardwareProfile {
  hardware: String,
  accelerator: String,
  expected_performance: String,
}

#[tauri::command]
fn detect_hardware() -> HardwareProfile {
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

#[tauri::command]
fn get_runtime_profile() -> RuntimeProfile {
  let hardware_profile = detect_hardware();
  let model_directory = resolve_model_directory();
  let cache_directory = resolve_cache_directory();

  let (llm_flags, image_flags) = match hardware_profile.accelerator.as_str() {
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
  };

  RuntimeProfile {
    hardware: hardware_profile.hardware,
    accelerator: hardware_profile.accelerator,
    expected_performance: hardware_profile.expected_performance,
    llm_flags,
    image_flags,
    model_directory,
    cache_directory,
    safety_prompt: SAFETY_PROMPT.to_string(),
    style_wrapper: STYLE_WRAPPER.to_string(),
  }
}

fn has_command(command: &str) -> bool {
  Command::new(command)
    .arg("--help")
    .output()
    .map(|result| result.status.success())
    .unwrap_or(false)
}

fn resolve_model_directory() -> String {
  if cfg!(target_os = "windows") {
    if let Ok(app_data) = env::var("APPDATA") {
      return PathBuf::from(app_data)
        .join("OmniPedia")
        .join("models")
        .display()
        .to_string();
    }
  }

  resolve_home_path(".local/share/OmniPedia/models")
}

fn resolve_cache_directory() -> String {
  if cfg!(target_os = "windows") {
    if let Ok(app_data) = env::var("APPDATA") {
      return PathBuf::from(app_data)
        .join("OmniPedia")
        .join("cache")
        .display()
        .to_string();
    }
  }

  resolve_home_path(".cache/OmniPedia")
}

fn resolve_home_path(relative_path: &str) -> String {
  env::var("HOME")
    .map(|home| PathBuf::from(home).join(relative_path).display().to_string())
    .unwrap_or_else(|_| relative_path.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![detect_hardware, get_runtime_profile])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn runtime_profile_includes_guardrails() {
    let profile = get_runtime_profile();
    assert!(profile.safety_prompt.contains("children encyclopedia"));
    assert!(profile.style_wrapper.contains("[SUBJECT]"));
  }

  #[test]
  fn runtime_profile_has_storage_paths() {
    let profile = get_runtime_profile();
    assert!(!profile.model_directory.is_empty());
    assert!(!profile.cache_directory.is_empty());
  }
}
