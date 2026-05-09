//! Serializable contracts exposed to the Vue frontend through Tauri commands.

use serde::{Deserialize, Serialize};

/// Full runtime profile displayed by the frontend status panel.
#[derive(Serialize)]
pub(crate) struct RuntimeProfile {
    pub(crate) hardware: String,
    pub(crate) accelerator: String,
    pub(crate) expected_performance: String,
    pub(crate) llm_flags: Vec<String>,
    pub(crate) image_flags: Vec<String>,
    pub(crate) model_directory: String,
    pub(crate) cache_directory: String,
    pub(crate) safety_prompt: String,
    pub(crate) style_wrapper: String,
    pub(crate) llm_binary: String,
    pub(crate) image_binary: String,
    pub(crate) llm_model: String,
    pub(crate) image_model: String,
    pub(crate) database_path: String,
}

/// Best-effort hardware and accelerator summary used to pick local inference flags.
#[derive(Serialize)]
pub(crate) struct HardwareProfile {
    pub(crate) hardware: String,
    pub(crate) accelerator: String,
    pub(crate) expected_performance: String,
}

/// Frontend request payload for text and image generation commands.
#[derive(Deserialize)]
pub(crate) struct ArticleRequest {
    pub(crate) article_id: String,
    pub(crate) locale: Option<String>,
    pub(crate) question: Option<String>,
}

/// Root catalog payload returned to the frontend.
#[derive(Serialize)]
pub(crate) struct CatalogResponse {
    pub(crate) domains: Vec<DomainDto>,
}

/// Encyclopedia domain with its localized metadata and sections.
#[derive(Serialize)]
pub(crate) struct DomainDto {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) icon: String,
    pub(crate) color: String,
    pub(crate) welcome: String,
    pub(crate) sections: Vec<SectionDto>,
}

/// Localized section entry pointing to an article.
#[derive(Serialize)]
pub(crate) struct SectionDto {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) icon: String,
    pub(crate) article_id: String,
}

/// Localized article metadata used by generation and rendering.
#[derive(Serialize)]
pub(crate) struct ArticleDto {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) summary: String,
    pub(crate) questions: Vec<String>,
}

/// Generated educational text and its provenance.
#[derive(Serialize)]
pub(crate) struct GeneratedArticle {
    pub(crate) article: ArticleDto,
    pub(crate) generated_text: String,
    pub(crate) source: String,
    pub(crate) prompt: String,
    pub(crate) cached: bool,
}

/// Generated or fallback illustration metadata and cache status.
#[derive(Serialize)]
pub(crate) struct GeneratedImage {
    pub(crate) article_id: String,
    pub(crate) prompt: String,
    pub(crate) source: String,
    pub(crate) image_path: String,
    pub(crate) cached: bool,
}

/// Availability report for models, sidecars, storage, and downloads.
#[derive(Serialize)]
pub(crate) struct ModelStatus {
    pub(crate) model_directory: String,
    pub(crate) cache_directory: String,
    pub(crate) llm_binary: String,
    pub(crate) image_binary: String,
    pub(crate) llm_model: String,
    pub(crate) image_model: String,
    pub(crate) llm_binary_ready: bool,
    pub(crate) image_binary_ready: bool,
    pub(crate) llm_model_ready: bool,
    pub(crate) image_model_ready: bool,
    pub(crate) llm_ready: bool,
    pub(crate) image_ready: bool,
    pub(crate) database_path: String,
    pub(crate) downloads: ModelPreparationStatus,
}

/// Preparation state for one local model asset.
#[derive(Clone, Serialize)]
pub(crate) struct ModelAssetStatus {
    pub(crate) label: String,
    pub(crate) file_name: String,
    pub(crate) path: String,
    pub(crate) url: String,
    pub(crate) ready: bool,
    pub(crate) downloaded: bool,
    pub(crate) error: Option<String>,
}

/// Combined preparation state for all optional local models.
#[derive(Clone, Serialize)]
pub(crate) struct ModelPreparationStatus {
    pub(crate) llm_binary: ModelAssetStatus,
    pub(crate) image_binary: ModelAssetStatus,
    pub(crate) llm: ModelAssetStatus,
    pub(crate) image: ModelAssetStatus,
}

/// Result of a local generation cache purge.
#[derive(Serialize)]
pub(crate) struct CachePurgeResult {
    pub(crate) entries_deleted: usize,
    pub(crate) files_deleted: usize,
}
