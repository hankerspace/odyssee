//! Tauri command handlers that coordinate storage, cache, and inference adapters.

use crate::generation::{
    build_article_prompt, fallback_article_text, image_prompt, now_millis, write_placeholder_svg,
};
use crate::hardware::{build_runtime_profile, detect_hardware_profile};
use crate::model_download::{
    get_model_status_for_paths, image_model_ready, prepare_models_for_paths,
};
use crate::models::{
    ArticleRequest, CachePurgeResult, CatalogResponse, GeneratedArticle, GeneratedImage, HardwareProfile,
    ModelPreparationStatus, ModelStatus, RuntimeProfile,
};
use crate::paths::{ensure_storage, executable_exists, image_sidecar_ready, RuntimePaths};
use crate::sidecars::{run_llama_sidecar, run_stable_diffusion_sidecar};
use crate::storage::{
    clear_cache, load_article, load_catalog, normalize_locale, open_database, read_cache, write_cache,
};
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

/// Attempts to prepare optional local sidecars and models when the frontend explicitly asks for it.
#[tauri::command]
pub(crate) fn prepare_models() -> Result<ModelPreparationStatus, String> {
    let paths = RuntimePaths::resolve();
    log::info!(
        "Runtime preparation requested under '{}'",
        paths.data_dir.display()
    );
    let status = prepare_models_for_paths(&paths)?;
    log::info!(
    "Runtime preparation finished: llm_binary_ready={}, llm_model_ready={}, image_binary_ready={}, image_model_ready={}",
    status.llm_binary.ready,
    status.llm.ready,
    status.image_binary.ready,
    status.image.ready
  );
    Ok(status)
}

/// Clears generated text/image cache entries and removes cached image files.
#[tauri::command]
pub(crate) fn clear_generation_cache() -> Result<CachePurgeResult, String> {
    let paths = RuntimePaths::resolve();
    let connection = open_database(&paths)?;
    let entries_deleted = clear_cache(&connection)?;
    let files_deleted = clear_image_cache_files(&paths.image_cache_dir)?;
    log::info!(
        "Generation cache cleared: entries_deleted={entries_deleted}, files_deleted={files_deleted}"
    );
    Ok(CachePurgeResult {
        entries_deleted,
        files_deleted,
    })
}

/// Loads the localized deterministic catalog from SQLite.
#[tauri::command]
pub(crate) fn get_catalog(locale: Option<String>) -> Result<CatalogResponse, String> {
    let paths = RuntimePaths::resolve();
    let connection = open_database(&paths)?;
    let language = normalize_locale(locale.as_deref());
    log::info!("Catalog requested for locale='{language}'");
    let catalog = load_catalog(&connection, language)?;
    log::info!(
        "Catalog loaded with {} domains for locale='{language}'",
        catalog.domains.len()
    );
    Ok(catalog)
}

/// Generates child-safe educational article text, using cache and offline fallback when needed.
#[tauri::command]
pub(crate) fn generate_article(request: ArticleRequest) -> Result<GeneratedArticle, String> {
    let paths = RuntimePaths::resolve();
    let connection = open_database(&paths)?;
    let language = normalize_locale(request.locale.as_deref());
    let article = load_article(&connection, &request.article_id, language)?;
    let curiosity_question = request
        .question
        .as_deref()
        .map(str::trim)
        .filter(|question| !question.is_empty())
        .filter(|question| {
            article
                .questions
                .iter()
                .any(|candidate| candidate.trim() == *question)
        })
        .map(str::to_string);
    log::info!(
        "Article generation requested: article_id='{}', locale='{language}', curiosity_question={}",
        article.id,
        curiosity_question.as_deref().unwrap_or("none")
    );
    let cache_key = curiosity_question.as_deref().map_or_else(
        || format!("text:{}:{}", language, article.id),
        |question| format!("text:{}:{}:question:{}", language, article.id, question),
    );

    if let Some(text) = read_cache(&connection, &cache_key)? {
        log::info!(
            "Article text cache hit: article_id='{}', locale='{language}'",
            article.id
        );
        return Ok(GeneratedArticle {
            article,
            generated_text: text,
            source: "cache".to_string(),
            prompt: String::new(),
            cached: true,
        });
    }

    let prompt = build_article_prompt(&article, language, curiosity_question.as_deref());
    let profile = get_runtime_profile();
    let llm_ready = executable_exists(&paths.llm_binary) && paths.llm_model.exists();
    let (generated_text, source) = if llm_ready {
        log::info!("Running llama.cpp sidecar for article_id='{}'", article.id);
        match run_llama_sidecar(&paths, &profile.llm_flags, &prompt) {
            Ok(text) => (text, "llama.cpp"),
            Err(error) => {
                log::warn!(
                    "llama.cpp sidecar failed for article_id='{}'; using fallback text: {error}",
                    article.id
                );
                (
                    fallback_article_text(&article, language, curiosity_question.as_deref(), Some(&error)),
                    "fallback",
                )
            }
        }
    } else {
        log::info!(
            "LLM sidecar or model missing for article_id='{}'; using fallback text",
            article.id
        );
        (
            fallback_article_text(&article, language, curiosity_question.as_deref(), None),
            "fallback",
        )
    };

    write_cache(&connection, &cache_key, &generated_text)?;
    log::info!(
        "Article text generated and cached: article_id='{}', locale='{language}'",
        article.id
    );

    Ok(GeneratedArticle {
        article,
        generated_text,
        source: source.to_string(),
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
    log::info!(
        "Image generation requested: article_id='{}', locale='{language}'",
        request.article_id
    );
    let article = load_article(&connection, &request.article_id, language)?;
    let cache_key = format!("image:{}:{}", language, article.id);
    let prompt = image_prompt(&article.title);

    if let Some(path) = read_cache(&connection, &cache_key)? {
        if Path::new(&path).exists() {
            log::info!(
                "Image cache hit: article_id='{}', locale='{language}', path='{path}'",
                article.id
            );
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
    let output_path = paths
        .image_cache_dir
        .join(format!("{}-{}.svg", article.id, now_millis()));
    let profile = get_runtime_profile();

    let source = if image_sidecar_ready(&paths) && image_model_ready(&paths.image_model) {
        let generated_path = output_path.with_extension("png");
        log::info!(
            "Running stable-diffusion.cpp sidecar for article_id='{}'",
            article.id
        );
        match run_stable_diffusion_sidecar(&paths, &profile.image_flags, &prompt, &generated_path) {
            Ok(()) => {
                write_cache(
                    &connection,
                    &cache_key,
                    &generated_path.display().to_string(),
                )?;
                log::info!(
                    "Image generated and cached: article_id='{}', path='{}'",
                    article.id,
                    generated_path.display()
                );
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
        log::info!(
            "Image sidecar or model missing for article_id='{}'; using SVG fallback",
            article.id
        );
        "fallback".to_string()
    };

    // The SVG fallback keeps no-model mode deterministic and fully offline.
    write_placeholder_svg(&output_path, &article, language)?;
    write_cache(&connection, &cache_key, &output_path.display().to_string())?;
    log::info!(
        "Fallback image generated and cached: article_id='{}', path='{}'",
        article.id,
        output_path.display()
    );

    Ok(GeneratedImage {
        article_id: article.id,
        prompt,
        source,
        image_path: output_path.display().to_string(),
        cached: false,
    })
}

fn clear_image_cache_files(image_cache_dir: &Path) -> Result<usize, String> {
    if !image_cache_dir.exists() {
        return Ok(0);
    }

    let mut files_deleted = 0;
    for entry in fs::read_dir(image_cache_dir).map_err(|error| error.to_string())? {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.is_file() {
            fs::remove_file(&path).map_err(|error| error.to_string())?;
            files_deleted += 1;
        }
    }
    Ok(files_deleted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear_image_cache_files_ignores_missing_directory() {
        let missing_dir = std::env::temp_dir().join(format!("odyssee-missing-cache-{}", now_millis()));

        let deleted = clear_image_cache_files(&missing_dir).expect("clear missing cache dir");

        assert_eq!(deleted, 0);
    }

    #[test]
    fn clear_image_cache_files_removes_only_files() {
        let cache_dir = std::env::temp_dir().join(format!("odyssee-cache-{}", now_millis()));
        let nested_dir = cache_dir.join("nested");
        fs::create_dir_all(&nested_dir).expect("create nested cache dir");
        fs::write(cache_dir.join("image.svg"), "svg").expect("write svg");
        fs::write(cache_dir.join("image.png"), "png").expect("write png");

        let deleted = clear_image_cache_files(&cache_dir).expect("clear image cache files");

        assert_eq!(deleted, 2);
        assert!(!cache_dir.join("image.svg").exists());
        assert!(!cache_dir.join("image.png").exists());
        assert!(nested_dir.exists());

        fs::remove_dir_all(&cache_dir).expect("remove cache dir");
    }
}
