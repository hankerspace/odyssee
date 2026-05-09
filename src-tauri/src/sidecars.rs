//! Sidecar process adapters for llama.cpp and stable-diffusion.cpp.

use crate::paths::RuntimePaths;
use std::env;
use std::path::Path;
use std::process::{Command, Stdio};

pub(crate) fn run_llama_sidecar(paths: &RuntimePaths, flags: &[String], prompt: &str) -> Result<String, String> {
  log::info!("Starting llama.cpp sidecar: binary='{}', model='{}'", paths.llm_binary.display(), paths.llm_model.display());
  let output = Command::new(&paths.llm_binary)
    .arg("-m")
    .arg(&paths.llm_model)
    .arg("-p")
    .arg(prompt)
    .arg("--single-turn")
    .arg("--no-display-prompt")
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
  let stdout = String::from_utf8_lossy(&output.stdout);
  let stderr = String::from_utf8_lossy(&output.stderr);
  clean_llama_output(&stdout, prompt)
    .or_else(|_| clean_llama_output(&stderr, prompt))
    .or_else(|_| clean_llama_output(&format!("{stdout}\n{stderr}"), prompt))
}

fn clean_llama_output(output: &str, prompt: &str) -> Result<String, String> {
  let answer = output
    .find(prompt)
    .map(|index| &output[index + prompt.len()..])
    .unwrap_or(output);
  let cleaned = answer
    .lines()
    .filter(|line| !is_llama_noise_line(line))
    .collect::<Vec<_>>()
    .join("\n")
    .trim()
    .to_string();

  if cleaned.is_empty() {
    Err("llama.cpp sidecar returned no usable text".to_string())
  } else {
    Ok(cleaned)
  }
}

fn is_llama_noise_line(line: &str) -> bool {
  let trimmed = line.trim();
  trimmed == ">"
    || trimmed == "Exiting..."
    || trimmed == "Loading model..."
    || trimmed.starts_with("[ Prompt:")
    || trimmed.starts_with("[Prompt:")
    || trimmed.starts_with("ggml_")
    || trimmed.starts_with("common_memory_breakdown_print:")
}

pub(crate) fn run_stable_diffusion_sidecar(paths: &RuntimePaths, flags: &[String], prompt: &str, output_path: &Path) -> Result<(), String> {
  log::info!(
    "Starting stable-diffusion.cpp sidecar: binary='{}', model='{}', output='{}'",
    paths.image_binary.display(),
    paths.image_model.display(),
    output_path.display()
  );
  let mut command = Command::new(&paths.image_binary);
  command
    .arg("-m")
    .arg(&paths.image_model)
    .arg("-p")
    .arg(prompt)
    .arg("-o")
    .arg(output_path)
    .args(flags)
    .stdin(Stdio::null());

  configure_stable_diffusion_library_path(&mut command, paths);

  let output = command.output()
    .map_err(|error| error.to_string())?;

  if output.status.success() && output_path.exists() {
    log::info!("stable-diffusion.cpp sidecar completed successfully");
    Ok(())
  } else {
    log::warn!("stable-diffusion.cpp sidecar failed with status {}", output.status);
    Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
  }
}

fn configure_stable_diffusion_library_path(command: &mut Command, paths: &RuntimePaths) {
  #[cfg(target_os = "macos")]
  {
    if let Some(directory) = paths.image_binary.parent() {
      let mut directories = vec![directory.to_path_buf()];
      if let Some(existing) = env::var_os("DYLD_LIBRARY_PATH") {
        directories.extend(env::split_paths(&existing));
      }
      if let Ok(joined) = env::join_paths(directories) {
        command.env("DYLD_LIBRARY_PATH", joined);
      }
    }
  }

  #[cfg(not(target_os = "macos"))]
  {
    let _ = command;
    let _ = paths;
  }
}

#[cfg(test)]
mod tests {
  use super::clean_llama_output;

  #[test]
  fn clean_llama_output_removes_prompt_markers() {
    let output = ">\n>\nLes renards arctiques changent de pelage selon les saisons.\n>\n";

    let cleaned = clean_llama_output(output, "Sujet: Renard arctique").expect("output should keep generated text");

    assert_eq!(
      cleaned,
      "Les renards arctiques changent de pelage selon les saisons."
    );
  }

  #[test]
  fn clean_llama_output_rejects_marker_only_output() {
    let error = clean_llama_output(">\n>\n>\n", "Sujet: Renard arctique").expect_err("marker-only output should fail");

    assert_eq!(error, "llama.cpp sidecar returned no usable text");
  }

  #[test]
  fn clean_llama_output_rejects_metal_diagnostic_only_output() {
    let output = "ggml_metal_device_init: tensor API disabled for pre-M5 and pre-A19 devices\n\
ggml_metal_library_init: using embedded metal library\n\
ggml_metal_device_init: GPU name: MTL0 (Apple M4 Max)\n\
common_memory_breakdown_print: | memory breakdown [MiB] | total free self model context compute unaccounted |\n\
ggml_metal_free: deallocating\n";

    let error = clean_llama_output(output, "Sujet: Mars Rover").expect_err("diagnostic-only output should fail");

    assert_eq!(error, "llama.cpp sidecar returned no usable text");
  }

  #[test]
  fn clean_llama_output_keeps_generated_answer_after_echoed_prompt() {
    let prompt = "You are a children encyclopedia.\n\nSujet: High-Speed Train\nRésumé fiable: A high-speed train uses powerful electric motors.";
    let output = format!(
      "Loading model...\n\n> {prompt}\n\nA high-speed train is a special train.\n\nCuriosity questions:\n\n- How fast can it go?\n- Why are tracks special?\n- How does it stay safe?\n\n[ Prompt: 1266.4 t/s | Generation: 131.1 t/s ]\n\nExiting...\n"
    );

    let cleaned = clean_llama_output(&output, prompt).expect("output should keep generated answer");

    assert_eq!(
      cleaned,
      "A high-speed train is a special train.\n\nCuriosity questions:\n\n- How fast can it go?\n- Why are tracks special?\n- How does it stay safe?"
    );
  }
}