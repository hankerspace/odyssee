//! Tauri command handlers that coordinate storage, cache, and inference adapters.

use crate::generation::{build_article_prompt, fallback_article_text, image_prompt, now_millis, write_placeholder_svg};
use crate::hardware::{build_runtime_profile, detect_hardware_profile};
use crate::model_download::{get_model_status_for_paths, prepare_models_for_paths};
use crate::models::{ArticleRequest, CatalogResponse, GeneratedArticle, GeneratedImage, HardwareProfile, ModelPreparationStatus, ModelStatus, RuntimeProfile};
use crate::paths::{ensure_storage, RuntimePaths};
use crate::sidecars::{run_llama_sidecar, run_stable_diffusion_sidecar};
use crate::storage::{load_article, load_catalog, normalize_locale, open_database, read_cache, write_cache};
use std::fs;
use std::path::Path;

/// Returns the detected local hardware profile for the frontend status panel.
#[tauri::command]
pub(crate) fn detect_hardware() -> HardwareProfile {
  let profile = detect_hardware_profile();
  log::info!(
    "Detected hardware profile: hardware='{}', accelerator='{}'",
    profile.hardware,
    profile.accelerator
  );
  profile
}

/// Returns the complete local runtime profile, including paths and inference flags.
#[tauri::command]
pub(crate) fn get_runtime_profile() -> RuntimeProfile {
  let profile = build_runtime_profile();
  log::info!(
    "Built runtime profile: accelerator='{}', model_directory='{}', cache_directory='{}'",
    profile.accelerator,
    profile.model_directory,
    profile.cache_directory
  );
  profile
}

/// Reports whether optional local sidecars and models are ready.
#[tauri::command]
pub(crate) fn get_model_status() -> Result<ModelStatus, String> {
  let paths = RuntimePaths::resolve();
  ensure_storage(&paths)?;

  let status = get_model_status_for_paths(&paths);
  log::info!(
    "Model status checked: llm_ready={}, image_ready={}, model_directory='{}'",
    status.llm_ready,
    status.image_ready,
    status.model_directory
  );
  Ok(status)
}

/// Attempts to prepare optional local models when the frontend explicitly asks for it.
#[tauri::command]
pub(crate) fn prepare_models() -> Result<ModelPreparationStatus, String> {
  let paths = RuntimePaths::resolve();
  log::info!("Model preparation requested for '{}'", paths.model_dir.display());
  let status = prepare_models_for_paths(&paths)?;
  log::info!(
    "Model preparation finished: llm_ready={}, image_ready={}",
    status.llm.ready,
    status.image.ready
  );
  Ok(status)
}

/// Loads the localized deterministic catalog from SQLite.
#[tauri::command]
pub(crate) fn get_catalog(locale: Option<String>) -> Result<CatalogResponse, String> {
  let paths = RuntimePaths::resolve();
  let connection = open_database(&paths)?;
  let language = normalize_locale(locale.as_deref());
  log::info!("Catalog requested for locale='{language}'");
  let catalog = load_catalog(&connection, language)?;
  log::info!("Catalog loaded with {} domains for locale='{language}'", catalog.domains.len());
  Ok(catalog)
}

/// Generates child-safe educational article text, using cache and offline fallback when needed.
#[tauri::command]
pub(crate) fn generate_article(request: ArticleRequest) -> Result<GeneratedArticle, String> {
  let paths = RuntimePaths::resolve();
  let connection = open_database(&paths)?;
  let language = normalize_locale(request.locale.as_deref());
  log::info!("Article generation requested: article_id='{}', locale='{language}'", request.article_id);
  let article = load_article(&connection, &request.article_id, language)?;
  let cache_key = format!("text:{}:{}", language, article.id);

  if let Some(text) = read_cache(&connection, &cache_key)? {
    log::info!("Article text cache hit: article_id='{}', locale='{language}'", article.id);
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
    log::info!("Running llama.cpp sidecar for article_id='{}'", article.id);
    run_llama_sidecar(&paths, &profile.llm_flags, &prompt).unwrap_or_else(|error| {
      log::warn!("llama.cpp sidecar failed for article_id='{}'; using fallback text: {error}", article.id);
      fallback_article_text(&article, language, Some(&error))
    })
  } else {
    log::info!("LLM sidecar or model missing for article_id='{}'; using fallback text", article.id);
    fallback_article_text(&article, language, None)
  };

  write_cache(&connection, &cache_key, &generated_text)?;
  log::info!("Article text generated and cached: article_id='{}', locale='{language}'", article.id);

  Ok(GeneratedArticle {
    article,
    generated_text,
    source: if llm_ready { "llama.cpp" } else { "fallback" }.to_string(),
    prompt,
    cached: false,
  })
}

/// Generates or retrieves an illustration, using deterministic SVG fallback when needed.
#[tauri::command]
pub(crate) fn generate_image(request: ArticleRequest) -> Result<GeneratedImage, String> {
  let paths = RuntimePaths::resolve();
  let connection = open_database(&paths)?;
  let language = normalize_locale(request.locale.as_deref());
  log::info!("Image generation requested: article_id='{}', locale='{language}'", request.article_id);
  let article = load_article(&connection, &request.article_id, language)?;
  let cache_key = format!("image:{}:{}", language, article.id);
  let prompt = image_prompt(&article.title);

  if let Some(path) = read_cache(&connection, &cache_key)? {
    if Path::new(&path).exists() {
      log::info!("Image cache hit: article_id='{}', locale='{language}', path='{path}'", article.id);
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
    log::info!("Running stable-diffusion.cpp sidecar for article_id='{}'", article.id);
    match run_stable_diffusion_sidecar(&paths, &profile.image_flags, &prompt, &generated_path) {
      Ok(()) => {
        write_cache(&connection, &cache_key, &generated_path.display().to_string())?;
        log::info!("Image generated and cached: article_id='{}', path='{}'", article.id, generated_path.display());
        return Ok(GeneratedImage {
          article_id: article.id,
          prompt,
          source: "stable-diffusion.cpp".to_string(),
          image_path: generated_path.display().to_string(),
          cached: false,
        });
      }
      Err(error) => {
        log::warn!("stable-diffusion.cpp sidecar failed for article_id='{}'; using SVG fallback: {error}", article.id);
        format!("fallback ({error})")
      }
    }
  } else {
    log::info!("Image sidecar or model missing for article_id='{}'; using SVG fallback", article.id);
    "fallback".to_string()
  };

  // The SVG fallback keeps no-model mode deterministic and fully offline.
  write_placeholder_svg(&output_path, &article, language)?;
  write_cache(&connection, &cache_key, &output_path.display().to_string())?;
  log::info!("Fallback image generated and cached: article_id='{}', path='{}'", article.id, output_path.display());

  Ok(GeneratedImage {
    article_id: article.id,
    prompt,
    source,
    image_path: output_path.display().to_string(),
    cached: false,
  })
}