//! Sidecar process adapters for llama.cpp and stable-diffusion.cpp.

use crate::constants::PNG_SIGNATURE;
use crate::paths::RuntimePaths;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};

const LLAMA_CONTROL_ARGS: [&str; 5] = ["--single-turn", "--no-display-prompt", "--simple-io", "-n", "420"];

pub(crate) fn run_llama_sidecar(paths: &RuntimePaths, flags: &[String], prompt: &str) -> Result<String, String> {
  log::info!("Starting llama.cpp sidecar: binary='{}', model='{}'", paths.llm_binary.display(), paths.llm_model.display());
  let output = Command::new(&paths.llm_binary)
    .arg("-m")
    .arg(&paths.llm_model)
    .arg("-p")
    .arg(prompt)
    .args(LLAMA_CONTROL_ARGS)
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
  let output = normalize_llama_line_breaks(&remove_backspace_sequences(output));
  let prompt = normalize_llama_line_breaks(&remove_backspace_sequences(prompt));
  let answer = extract_llama_answer(&output, &prompt);
  let cleaned = answer
    .lines()
    .map(trim_llama_diagnostic_fragments)
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

fn normalize_llama_line_breaks(value: &str) -> String {
  value.replace("\r\n", "\n").replace('\r', "\n")
}

fn extract_llama_answer(output: &str, prompt: &str) -> String {
  if let Some(index) = output.find(prompt) {
    return output[index + prompt.len()..].to_string();
  }

  let prompt_line_count = prompt.lines().count();
  let lines = output.lines().collect::<Vec<_>>();
  lines
    .iter()
    .position(|line| line.trim_start().starts_with("> "))
    .map(|index| lines[(index + prompt_line_count).min(lines.len())..].join("\n"))
    .unwrap_or_else(|| output.to_string())
}

fn remove_backspace_sequences(value: &str) -> String {
  let mut cleaned = Vec::new();
  for character in value.chars() {
    if character == '\u{8}' {
      cleaned.pop();
    } else {
      cleaned.push(character);
    }
  }
  cleaned.into_iter().collect()
}

fn trim_llama_diagnostic_fragments(line: &str) -> &str {
  ["common_memory_breakdown_print:", "ggml_metal_free:"]
    .iter()
    .filter_map(|marker| line.find(marker))
    .min()
    .map(|index| &line[..index])
    .unwrap_or(line)
}

fn is_llama_noise_line(line: &str) -> bool {
  let trimmed = line.trim();
  trimmed == ">"
    || trimmed == "Exiting..."
    || trimmed == "Loading model..."
    || trimmed.starts_with("Loading model...")
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

  if output.status.success() {
    validate_generated_png(output_path)?;
    log::info!("stable-diffusion.cpp sidecar completed successfully");
    Ok(())
  } else {
    log::warn!("stable-diffusion.cpp sidecar failed with status {}", output.status);
    Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
  }
}

fn validate_generated_png(output_path: &Path) -> Result<(), String> {
  let mut header = [0; 24];
  File::open(output_path)
    .and_then(|mut file| file.read_exact(&mut header))
    .map_err(|_| "stable-diffusion.cpp returned no usable image".to_string())?;

  if header[..8] != PNG_SIGNATURE || header[8..12] != [0, 0, 0, 13] || header[12..16] != *b"IHDR" {
    return Err("stable-diffusion.cpp returned an invalid PNG image".to_string());
  }

  let width = u32::from_be_bytes([header[16], header[17], header[18], header[19]]);
  let height = u32::from_be_bytes([header[20], header[21], header[22], header[23]]);
  if width == 0 || height == 0 {
    return Err("stable-diffusion.cpp returned an empty image".to_string());
  }

  Ok(())
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
  use super::{clean_llama_output, validate_generated_png, LLAMA_CONTROL_ARGS};
  use std::fs;
  use std::path::PathBuf;

  #[test]
  fn llama_control_args_do_not_use_removed_conversation_argument() {
    assert!(!LLAMA_CONTROL_ARGS.contains(&"--no-conversation"));
  }

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

  #[test]
  fn clean_llama_output_keeps_answer_after_interactive_prompt_with_crlf() {
    let prompt = "You are a children encyclopedia. Never mention violent, political, or inappropriate content. Stay factual and kind.\n\nAnswer in English for a child aged 6 to 10. Use short paragraphs and end with exactly three curiosity questions.\n\nSujet: The Sun\nRésumé fiable: The Sun is a star. It gives Earth light and heat, helping plants grow and life exist.\nQuestions proposées: How hot is it? | Why does it look yellow? | What is solar wind?";
    let echoed_prompt = prompt.replace('\n', "\r\n");
    let output = format!(
      "Loading model...\n\navailable commands:\n  /exit or Ctrl+C     stop or exit\n\n> {echoed_prompt}\n\nThe Sun is a star that is very far away from us.\n\nHow hot is it?\nWhy does it look yellow?\nWhat is solar wind?\n\n[ Prompt: 1273.0 t/s | Generation: 133.2 t/s ]\n\nExiting...\n"
    );

    let cleaned = clean_llama_output(&output, prompt).expect("output should keep generated answer");

    assert_eq!(
      cleaned,
      "The Sun is a star that is very far away from us.\n\nHow hot is it?\nWhy does it look yellow?\nWhat is solar wind?"
    );
  }

  #[test]
  fn clean_llama_output_keeps_answer_with_spinner_and_inline_diagnostics() {
    let prompt = "You are a children encyclopedia.\n\nSujet: The Sun\nRésumé fiable: The Sun is a star.";
    let output = format!(
      "Loading model... |\u{8}-\u{8}\\\u{8}|\u{8}  \u{8}\n\n> {prompt}\n\n|\u{8} \u{8}The Sun is a star that gives Earth light and heat.common_memory_breakdown_print: | memory breakdown [MiB] |\nggml_metal_free: deallocating\n\n[ Prompt: 1270.2 t/s | Generation: 129.0 t/s ]\n\nExiting...\n"
    );

    let cleaned = clean_llama_output(&output, &prompt).expect("output should keep generated answer");

    assert_eq!(
      cleaned,
      "The Sun is a star that gives Earth light and heat."
    );
  }

  #[test]
  fn clean_llama_output_keeps_questions_after_carriage_return_progress() {
    let prompt = "You are an offline children encyclopedia assistant.\n\nÉcris exactement trois questions.\n\nSujet: Égypte\nRésumé fiable: L'Égypte antique a construit des pyramides près du Nil.";
    let output = format!(
      "Loading model...\r> {prompt}\rPourquoi le Nil était-il important ?\rComment construisait-on les pyramides ?\rQue racontent les hiéroglyphes ?\r[ Prompt: 1260.0 t/s | Generation: 120.0 t/s ]\rExiting...\r"
    );

    let cleaned = clean_llama_output(&output, prompt).expect("output should keep generated questions");

    assert_eq!(
      cleaned,
      "Pourquoi le Nil était-il important ?\nComment construisait-on les pyramides ?\nQue racontent les hiéroglyphes ?"
    );
  }

  #[test]
  fn validate_generated_png_rejects_empty_file() {
    let output_path = temporary_test_path("empty.png");
    fs::write(&output_path, []).expect("empty test image should be written");

    let error = validate_generated_png(&output_path).expect_err("empty image should fail validation");

    assert_eq!(error, "stable-diffusion.cpp returned no usable image");
    fs::remove_file(output_path).ok();
  }

  #[test]
  fn validate_generated_png_accepts_png_with_dimensions() {
    let output_path = temporary_test_path("valid.png");
    let png_header = [
      0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n',
      0x00, 0x00, 0x00, 0x0d,
      b'I', b'H', b'D', b'R',
      0x00, 0x00, 0x00, 0x01,
      0x00, 0x00, 0x00, 0x01,
    ];
    fs::write(&output_path, png_header).expect("valid test image should be written");

    validate_generated_png(&output_path).expect("valid PNG header should pass validation");
    fs::remove_file(output_path).ok();
  }

  fn temporary_test_path(file_name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("odyssee-sidecars-{}-{file_name}", std::process::id()))
  }
}