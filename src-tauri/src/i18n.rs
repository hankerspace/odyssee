//! Backend translations for prompts and user-facing errors.

use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct BackendMessages {
  pub(crate) prompts: PromptMessages,
  pub(crate) errors: ErrorMessages,
  pub(crate) hardware: HardwareMessages,
}

#[derive(Deserialize)]
pub(crate) struct PromptMessages {
  pub(crate) article_instruction: String,
  pub(crate) curiosity_focus: String,
  pub(crate) questions_instruction: String,
  pub(crate) subcategories_instruction: String,
  pub(crate) age_instruction_3_6: String,
  pub(crate) age_instruction_6_10: String,
  pub(crate) age_instruction_10_14: String,
  pub(crate) article_subject_label: String,
  pub(crate) reliable_summary_label: String,
  pub(crate) safe_examples_label: String,
  pub(crate) category_label: String,
  pub(crate) reliable_welcome_label: String,
  pub(crate) existing_subcategories_label: String,
}

#[derive(Deserialize)]
pub(crate) struct ErrorMessages {
  pub(crate) unknown_domain: String,
  pub(crate) unknown_article: String,
  pub(crate) missing_llm: String,
  pub(crate) missing_llm_domain: String,
  pub(crate) missing_image_engine: String,
}

#[derive(Deserialize)]
pub(crate) struct HardwareMessages {
  pub(crate) very_smooth: String,
  pub(crate) near_instant: String,
  pub(crate) good_fluidity: String,
  pub(crate) pc_without_gpu: String,
  pub(crate) cpu_performance: String,
}

const FR_MESSAGES: &str = include_str!("../i18n/fr.json");
const EN_MESSAGES: &str = include_str!("../i18n/en.json");

pub(crate) fn backend_messages(locale: &str) -> BackendMessages {
  let content = if locale == "en" { EN_MESSAGES } else { FR_MESSAGES };

  serde_json::from_str(content).expect("backend i18n file must be valid")
}

pub(crate) fn translate(template: &str, values: &[(&str, &str)]) -> String {
  values.iter().fold(template.to_string(), |message, (key, value)| {
    message.replace(&format!("{{{key}}}"), value)
  })
}
