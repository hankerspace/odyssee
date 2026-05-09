//! Backend translations for user-facing errors and labels.

use crate::constants::{
  EN_BACKEND_MESSAGES, EN_PROMPT_MESSAGES, FR_BACKEND_MESSAGES, FR_PROMPT_MESSAGES,
  PromptMessages,
};

use serde::Deserialize;

pub(crate) struct BackendMessages {
  pub(crate) prompts: PromptMessages,
  pub(crate) errors: ErrorMessages,
  pub(crate) hardware: HardwareMessages,
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

#[derive(Deserialize)]
struct BackendLocaleMessages {
  errors: ErrorMessages,
  hardware: HardwareMessages,
}

pub(crate) fn backend_messages(locale: &str) -> BackendMessages {
  let content = if locale == "en" { EN_BACKEND_MESSAGES } else { FR_BACKEND_MESSAGES };
  let messages: BackendLocaleMessages = serde_json::from_str(content).expect("backend i18n file must be valid");

  BackendMessages {
    prompts: if locale == "en" { EN_PROMPT_MESSAGES } else { FR_PROMPT_MESSAGES },
    errors: messages.errors,
    hardware: messages.hardware,
  }
}

pub(crate) fn translate(template: &str, values: &[(&str, &str)]) -> String {
  values.iter().fold(template.to_string(), |message, (key, value)| {
    message.replace(&format!("{{{key}}}"), value)
  })
}
