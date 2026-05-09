//! Serializable contracts exposed to the Vue frontend through Tauri commands.

use serde::{Deserialize, Serialize};

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

#[derive(Serialize)]
pub(crate) struct HardwareProfile {
  pub(crate) hardware: String,
  pub(crate) accelerator: String,
  pub(crate) expected_performance: String,
}

#[derive(Deserialize)]
pub(crate) struct ArticleRequest {
  pub(crate) article_id: String,
  pub(crate) locale: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct CatalogResponse {
  pub(crate) domains: Vec<DomainDto>,
}

#[derive(Serialize)]
pub(crate) struct DomainDto {
  pub(crate) id: String,
  pub(crate) name: String,
  pub(crate) icon: String,
  pub(crate) color: String,
  pub(crate) welcome: String,
  pub(crate) sections: Vec<SectionDto>,
}

#[derive(Serialize)]
pub(crate) struct SectionDto {
  pub(crate) id: String,
  pub(crate) name: String,
  pub(crate) icon: String,
  pub(crate) article_id: String,
}

#[derive(Serialize)]
pub(crate) struct ArticleDto {
  pub(crate) id: String,
  pub(crate) title: String,
  pub(crate) summary: String,
  pub(crate) questions: Vec<String>,
}

#[derive(Serialize)]
pub(crate) struct GeneratedArticle {
  pub(crate) article: ArticleDto,
  pub(crate) generated_text: String,
  pub(crate) source: String,
  pub(crate) prompt: String,
  pub(crate) cached: bool,
}

#[derive(Serialize)]
pub(crate) struct GeneratedImage {
  pub(crate) article_id: String,
  pub(crate) prompt: String,
  pub(crate) source: String,
  pub(crate) image_path: String,
  pub(crate) cached: bool,
}

#[derive(Serialize)]
pub(crate) struct ModelStatus {
  pub(crate) model_directory: String,
  pub(crate) cache_directory: String,
  pub(crate) llm_binary: String,
  pub(crate) image_binary: String,
  pub(crate) llm_model: String,
  pub(crate) image_model: String,
  pub(crate) llm_ready: bool,
  pub(crate) image_ready: bool,
  pub(crate) database_path: String,
}