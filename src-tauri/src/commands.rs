//! Tauri command handlers that coordinate storage, cache, and inference adapters.

use crate::generation::{build_article_prompt, fallback_article_text, image_prompt, now_millis, write_placeholder_svg};
use crate::hardware::{build_runtime_profile, detect_hardware_profile};
use crate::models::{ArticleRequest, CatalogResponse, GeneratedArticle, GeneratedImage, HardwareProfile, ModelStatus, RuntimeProfile};
use crate::paths::{ensure_storage, RuntimePaths};
use crate::sidecars::{run_llama_sidecar, run_stable_diffusion_sidecar};
use crate::storage::{load_article, load_catalog, normalize_locale, open_database, read_cache, write_cache};
use std::fs;
use std::path::Path;

#[tauri::command]
pub(crate) fn detect_hardware() -> HardwareProfile {
  detect_hardware_profile()
}

#[tauri::command]
pub(crate) fn get_runtime_profile() -> RuntimeProfile {
  build_runtime_profile()
}

#[tauri::command]
pub(crate) fn get_model_status() -> Result<ModelStatus, String> {
  let paths = RuntimePaths::resolve();
  ensure_storage(&paths)?;

  Ok(ModelStatus {
    model_directory: paths.model_dir.display().to_string(),
    cache_directory: paths.cache_dir.display().to_string(),
    llm_binary: paths.llm_binary.display().to_string(),
    image_binary: paths.image_binary.display().to_string(),
    llm_model: paths.llm_model.display().to_string(),
    image_model: paths.image_model.display().to_string(),
    llm_ready: paths.llm_binary.exists() && paths.llm_model.exists(),
    image_ready: paths.image_binary.exists() && paths.image_model.exists(),
    database_path: paths.database_path.display().to_string(),
  })
}

#[tauri::command]
pub(crate) fn get_catalog(locale: Option<String>) -> Result<CatalogResponse, String> {
  let paths = RuntimePaths::resolve();
  let connection = open_database(&paths)?;
  let language = normalize_locale(locale.as_deref());
  load_catalog(&connection, language)
}

#[tauri::command]
pub(crate) fn generate_article(request: ArticleRequest) -> Result<GeneratedArticle, String> {
  let paths = RuntimePaths::resolve();
  let connection = open_database(&paths)?;
  let language = normalize_locale(request.locale.as_deref());
  let article = load_article(&connection, &request.article_id, language)?;
  let cache_key = format!("text:{}:{}", language, article.id);

  if let Some(text) = read_cache(&connection, &cache_key)? {
    return Ok(GeneratedArticle {
      article,
      generated_text: text,
      source: "cache".to_string(),
      prompt: String::new(),
      cached: true,
    });
  }

  let prompt = build_article_prompt(&article, language);
  let profile = get_runtime_profile();
  let llm_ready = paths.llm_binary.exists() && paths.llm_model.exists();
  let generated_text = if llm_ready {
    run_llama_sidecar(&paths, &profile.llm_flags, &prompt).unwrap_or_else(|error| fallback_article_text(&article, language, Some(&error)))
  } else {
    fallback_article_text(&article, language, None)
  };

  write_cache(&connection, &cache_key, &generated_text)?;

  Ok(GeneratedArticle {
    article,
    generated_text,
    source: if llm_ready { "llama.cpp" } else { "fallback" }.to_string(),
    prompt,
    cached: false,
  })
}

#[tauri::command]
pub(crate) fn generate_image(request: ArticleRequest) -> Result<GeneratedImage, String> {
  let paths = RuntimePaths::resolve();
  let connection = open_database(&paths)?;
  let language = normalize_locale(request.locale.as_deref());
  let article = load_article(&connection, &request.article_id, language)?;
  let cache_key = format!("image:{}:{}", language, article.id);
  let prompt = image_prompt(&article.title);

  if let Some(path) = read_cache(&connection, &cache_key)? {
    if Path::new(&path).exists() {
      return Ok(GeneratedImage {
        article_id: article.id,
        prompt,
        source: "cache".to_string(),
        image_path: path,
        cached: true,
      });
    }
  }

  fs::create_dir_all(&paths.image_cache_dir).map_err(|error| error.to_string())?;
  let output_path = paths.image_cache_dir.join(format!("{}-{}.svg", article.id, now_millis()));
  let profile = get_runtime_profile();

  let source = if paths.image_binary.exists() && paths.image_model.exists() {
    let generated_path = output_path.with_extension("png");
    match run_stable_diffusion_sidecar(&paths, &profile.image_flags, &prompt, &generated_path) {
      Ok(()) => {
        write_cache(&connection, &cache_key, &generated_path.display().to_string())?;
        return Ok(GeneratedImage {
          article_id: article.id,
          prompt,
          source: "stable-diffusion.cpp".to_string(),
          image_path: generated_path.display().to_string(),
          cached: false,
        });
      }
      Err(error) => format!("fallback ({error})"),
    }
  } else {
    "fallback".to_string()
  };

  write_placeholder_svg(&output_path, &article, language)?;
  write_cache(&connection, &cache_key, &output_path.display().to_string())?;

  Ok(GeneratedImage {
    article_id: article.id,
    prompt,
    source,
    image_path: output_path.display().to_string(),
    cached: false,
  })
}