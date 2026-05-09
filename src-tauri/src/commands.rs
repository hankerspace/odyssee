//! Tauri command handlers that coordinate storage, cache, and inference adapters.

use crate::generation::{
    build_article_prompt, build_questions_prompt, build_subcategories_prompt, image_prompt,
    normalize_age_range, now_millis, parse_generated_list,
};
use crate::hardware::{build_runtime_profile, detect_hardware_profile};
use crate::i18n::{backend_messages, translate};
use crate::model_download::{
    get_model_status_for_paths, image_model_ready, prepare_models_for_paths,
};
use crate::models::{
    ArticleRequest, CachePurgeResult, CatalogResponse, DomainRequest, GeneratedArticle,
    GeneratedImage, GeneratedQuestions, GeneratedSubcategories, HardwareProfile,
    ModelPreparationStatus, ModelStatus, RuntimeProfile,
};
use crate::paths::{ensure_storage, executable_exists, image_sidecar_ready, RuntimePaths};
use crate::sidecars::{run_llama_sidecar, run_stable_diffusion_sidecar};
use crate::storage::{
    clear_cache, load_article, load_catalog, load_domain_sections, normalize_locale, open_database,
    read_cache, write_cache,
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
pub(crate) fn get_runtime_profile(locale: Option<String>) -> RuntimeProfile {
    let language = normalize_locale(locale.as_deref());
    let profile = build_runtime_profile(language);
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
pub(crate) async fn get_model_status() -> Result<ModelStatus, String> {
    tauri::async_runtime::spawn_blocking(|| {
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
    })
    .await
    .map_err(|error| error.to_string())?
}

/// Attempts to prepare optional local sidecars and models when the frontend explicitly asks for it.
#[tauri::command]
pub(crate) async fn prepare_models() -> Result<ModelPreparationStatus, String> {
    tauri::async_runtime::spawn_blocking(|| {
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
    })
    .await
    .map_err(|error| error.to_string())?
}

/// Clears generated text/image cache entries and removes cached image files.
#[tauri::command]
pub(crate) async fn clear_generation_cache() -> Result<CachePurgeResult, String> {
    tauri::async_runtime::spawn_blocking(|| {
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
    })
    .await
    .map_err(|error| error.to_string())?
}

/// Loads the localized deterministic catalog from SQLite.
#[tauri::command]
pub(crate) async fn get_catalog(locale: Option<String>) -> Result<CatalogResponse, String> {
    tauri::async_runtime::spawn_blocking(move || {
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
    })
    .await
    .map_err(|error| error.to_string())?
}

/// Generates child-safe educational article text with the local LLM sidecar.
#[tauri::command]
pub(crate) async fn generate_article(request: ArticleRequest) -> Result<GeneratedArticle, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let paths = RuntimePaths::resolve();
        let connection = open_database(&paths)?;
        let language = normalize_locale(request.locale.as_deref());
        let age_range = normalize_age_range(request.age_range.as_deref());
        let article = load_article(&connection, &request.article_id, language)?;
        let generated_questions_cache_key = format!("questions:{}:{}:{}", language, age_range, article.id);
        let generated_questions = read_cache(&connection, &generated_questions_cache_key)?.unwrap_or_default();
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
                    || generated_questions
                        .lines()
                        .any(|candidate| candidate.trim() == *question)
            })
            .map(str::to_string);
        log::info!(
            "Article generation requested: article_id='{}', locale='{language}', age_range='{age_range}', curiosity_question={}",
            article.id,
            curiosity_question.as_deref().unwrap_or("none")
        );
        let cache_key = curiosity_question.as_deref().map_or_else(
            || format!("text:{}:{}:{}", language, age_range, article.id),
            |question| format!("text:{}:{}:{}:question:{}", language, age_range, article.id, question),
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

        let prompt = build_article_prompt(&article, language, age_range, curiosity_question.as_deref());
        let profile = build_runtime_profile(language);
        let llm_ready = executable_exists(&paths.llm_binary) && paths.llm_model.exists();
        if !llm_ready {
            return Err(translate(
                &backend_messages(language).errors.missing_llm,
                &[("article_id", &article.id)],
            ));
        }

        log::info!("Running llama.cpp sidecar for article_id='{}'", article.id);
        let generated_text = run_llama_sidecar(&paths, &profile.llm_flags, &prompt)
            .map_err(|error| format!("llama.cpp sidecar failed for article_id='{}': {error}", article.id))?;

        write_cache(&connection, &cache_key, &generated_text)?;
        log::info!(
            "Article text generated and cached: article_id='{}', locale='{language}'",
            article.id
        );

        Ok(GeneratedArticle {
            article,
            generated_text,
            source: "llama.cpp".to_string(),
            prompt,
            cached: false,
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

/// Generates child-safe follow-up questions with a dedicated LLM call.
#[tauri::command]
pub(crate) async fn generate_questions(request: ArticleRequest) -> Result<GeneratedQuestions, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let paths = RuntimePaths::resolve();
        let connection = open_database(&paths)?;
        let language = normalize_locale(request.locale.as_deref());
        let age_range = normalize_age_range(request.age_range.as_deref());
        let article = load_article(&connection, &request.article_id, language)?;
        log::info!(
            "Question generation requested: article_id='{}', locale='{language}', age_range='{age_range}'",
            article.id
        );
        let cache_key = format!("questions:{}:{}:{}", language, age_range, article.id);
        let prompt = build_questions_prompt(&article, language, age_range);

        if let Some(value) = read_cache(&connection, &cache_key)? {
            return Ok(GeneratedQuestions {
                article_id: article.id,
                questions: value.lines().map(str::to_string).collect(),
                source: "cache".to_string(),
                prompt,
                cached: true,
            });
        }

        let profile = build_runtime_profile(language);
        let llm_ready = executable_exists(&paths.llm_binary) && paths.llm_model.exists();
        if !llm_ready {
            return Err(translate(
                &backend_messages(language).errors.missing_llm,
                &[("article_id", &article.id)],
            ));
        }

        log::info!("Running llama.cpp sidecar for questions: article_id='{}'", article.id);
        let text = run_llama_sidecar(&paths, &profile.llm_flags, &prompt)
            .map_err(|error| format!("llama.cpp questions sidecar failed for article_id='{}': {error}", article.id))?;
        let questions = parse_generated_list(&text, 3);
        if questions.len() != 3 {
            return Err(format!(
                "llama.cpp returned {} question(s) for article_id='{}', expected 3",
                questions.len(),
                article.id
            ));
        }

        write_cache(&connection, &cache_key, &questions.join("\n"))?;
        Ok(GeneratedQuestions {
            article_id: article.id,
            questions,
            source: "llama.cpp".to_string(),
            prompt,
            cached: false,
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

/// Generates child-safe subcategories for a domain with a dedicated LLM call.
#[tauri::command]
pub(crate) async fn generate_subcategories(request: DomainRequest) -> Result<GeneratedSubcategories, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let paths = RuntimePaths::resolve();
        let connection = open_database(&paths)?;
        let language = normalize_locale(request.locale.as_deref());
        let age_range = normalize_age_range(request.age_range.as_deref());
        let catalog = load_catalog(&connection, language)?;
        let domain = catalog
            .domains
            .into_iter()
            .find(|domain| domain.id == request.domain_id)
            .ok_or_else(|| {
                translate(
                    &backend_messages(language).errors.unknown_domain,
                    &[("domain_id", request.domain_id.as_str())],
                )
            })?;
        let base_sections = load_domain_sections(&connection, &domain.id, language)?;
        log::info!(
            "Subcategory generation requested: domain_id='{}', locale='{language}', age_range='{age_range}'",
            domain.id
        );
        let cache_key = format!("subcategories:{}:{}:{}", language, age_range, domain.id);
        let visit_key = format!("subcategories-visits:{}:{}:{}", language, age_range, domain.id);
        let visit_index = next_subcategory_visit_index(&connection, &visit_key)?;
        let prompt = build_subcategories_prompt(&domain, language, age_range);

        if let Some(value) = read_cache(&connection, &cache_key)? {
            let labels = parse_generated_list(&value, base_sections.len());
            let sections = rotate_sections(
                merge_generated_subcategory_labels(&base_sections, &labels),
                visit_index,
            );
            return Ok(GeneratedSubcategories {
                domain_id: domain.id,
                sections,
                source: "cache".to_string(),
                prompt,
                cached: true,
            });
        }

        let profile = build_runtime_profile(language);
        let llm_ready = executable_exists(&paths.llm_binary) && paths.llm_model.exists();
        if !llm_ready {
            return Err(translate(
                &backend_messages(language).errors.missing_llm_domain,
                &[("domain_id", &domain.id)],
            ));
        }

        log::info!("Running llama.cpp sidecar for subcategories: domain_id='{}'", domain.id);
        let text = match run_llama_sidecar(&paths, &profile.llm_flags, &prompt) {
            Ok(text) => text,
            Err(error) => {
                log::warn!(
                    "llama.cpp subcategory sidecar failed for domain_id='{}': {error}. Falling back to catalog subcategories.",
                    domain.id
                );
                return Ok(catalog_subcategory_response(
                    domain.id,
                    base_sections,
                    prompt,
                    visit_index,
                ));
            }
        };
        let labels = parse_generated_list(&text, base_sections.len());
        if labels.len() != base_sections.len() {
            log::warn!(
                "llama.cpp returned {} subcategorie(s) for domain_id='{}', expected {}. Falling back to catalog subcategories.",
                labels.len(),
                domain.id,
                base_sections.len()
            );
            return Ok(catalog_subcategory_response(
                domain.id,
                base_sections,
                prompt,
                visit_index,
            ));
        }
        let sections = rotate_sections(
            merge_generated_subcategory_labels(&base_sections, &labels),
            visit_index,
        );

        write_cache(&connection, &cache_key, &labels.join("\n"))?;
        Ok(GeneratedSubcategories {
            domain_id: domain.id,
            sections,
            source: "llama.cpp".to_string(),
            prompt,
            cached: false,
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

fn catalog_subcategory_response(
    domain_id: String,
    sections: Vec<crate::models::SectionDto>,
    prompt: String,
    visit_index: usize,
) -> GeneratedSubcategories {
    GeneratedSubcategories {
        domain_id,
        sections: rotate_sections(sections, visit_index),
        source: "catalog".to_string(),
        prompt,
        cached: false,
    }
}

fn merge_generated_subcategory_labels(
    base_sections: &[crate::models::SectionDto],
    labels: &[String],
) -> Vec<crate::models::SectionDto> {
    base_sections
        .iter()
        .zip(labels.iter())
        .map(|(section, label)| crate::models::SectionDto {
            id: section.id.clone(),
            name: label.clone(),
            icon: section.icon.clone(),
            article_id: section.article_id.clone(),
        })
        .collect()
}

fn rotate_sections(
    mut sections: Vec<crate::models::SectionDto>,
    visit_index: usize,
) -> Vec<crate::models::SectionDto> {
    if sections.len() > 1 {
        let offset = visit_index % sections.len();
        sections.rotate_left(offset);
    }
    sections
}

fn next_subcategory_visit_index(
    connection: &rusqlite::Connection,
    visit_key: &str,
) -> Result<usize, String> {
    let visit_index = read_cache(connection, visit_key)?
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0);
    write_cache(connection, visit_key, &(visit_index + 1).to_string())?;
    Ok(visit_index)
}

/// Generates or retrieves an illustration with the local image sidecar.
#[tauri::command]
pub(crate) async fn generate_image(request: ArticleRequest) -> Result<GeneratedImage, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let paths = RuntimePaths::resolve();
        let connection = open_database(&paths)?;
        let language = normalize_locale(request.locale.as_deref());
        let age_range = normalize_age_range(request.age_range.as_deref());
        log::info!(
            "Image generation requested: article_id='{}', locale='{language}', age_range='{age_range}'",
            request.article_id
        );
        let article = load_article(&connection, &request.article_id, language)?;
        let cache_key = format!("image:{}:{}:{}", language, age_range, article.id);
        let prompt = image_prompt(&article.title, language, age_range);

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
            .join(format!("{}-{}.png", article.id, now_millis()));
        let profile = build_runtime_profile(language);

        if !image_sidecar_ready(&paths) || !image_model_ready(&paths.image_model) {
            return Err(translate(
                &backend_messages(language).errors.missing_image_engine,
                &[("article_id", &article.id)],
            ));
        }

        log::info!(
            "Running stable-diffusion.cpp sidecar for article_id='{}'",
            article.id
        );
        run_stable_diffusion_sidecar(&paths, &profile.image_flags, &prompt, &output_path)
            .map_err(|error| format!("stable-diffusion.cpp sidecar failed for article_id='{}': {error}", article.id))?;
        write_cache(&connection, &cache_key, &output_path.display().to_string())?;
        log::info!(
            "Image generated and cached: article_id='{}', path='{}'",
            article.id,
            output_path.display()
        );

        Ok(GeneratedImage {
            article_id: article.id,
            prompt,
            source: "stable-diffusion.cpp".to_string(),
            image_path: output_path.display().to_string(),
            cached: false,
        })
    })
    .await
    .map_err(|error| error.to_string())?
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

    fn sample_sections() -> Vec<crate::models::SectionDto> {
        vec![
            crate::models::SectionDto {
                id: "animals".to_string(),
                name: "Animaux".to_string(),
                icon: "PawPrint".to_string(),
                article_id: "arctic-fox".to_string(),
            },
            crate::models::SectionDto {
                id: "oceans".to_string(),
                name: "Océans".to_string(),
                icon: "Waves".to_string(),
                article_id: "coral-reef".to_string(),
            },
        ]
    }

    #[test]
    fn catalog_subcategory_response_uses_existing_sections() {
        let response = catalog_subcategory_response(
            "nature".to_string(),
            sample_sections(),
            "Prompt sous-catégories".to_string(),
            0,
        );

        assert_eq!(response.domain_id, "nature");
        assert_eq!(response.sections.len(), 2);
        assert_eq!(response.sections[0].name, "Animaux");
        assert_eq!(response.sections[1].name, "Océans");
        assert_eq!(response.source, "catalog");
        assert_eq!(response.prompt, "Prompt sous-catégories");
        assert!(!response.cached);
    }

    #[test]
    fn catalog_subcategory_response_rotates_between_visits() {
        let first = catalog_subcategory_response(
            "nature".to_string(),
            sample_sections(),
            "Prompt sous-catégories".to_string(),
            0,
        );
        let second = catalog_subcategory_response(
            "nature".to_string(),
            sample_sections(),
            "Prompt sous-catégories".to_string(),
            1,
        );

        let first_ids: Vec<&str> = first
            .sections
            .iter()
            .map(|section| section.id.as_str())
            .collect();
        let second_ids: Vec<&str> = second
            .sections
            .iter()
            .map(|section| section.id.as_str())
            .collect();

        assert_ne!(first_ids, second_ids);
    }

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
