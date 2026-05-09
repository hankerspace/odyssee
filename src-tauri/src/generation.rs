//! Prompt construction for local generation.

use crate::constants::{
  AGE_RANGE_10_14, AGE_RANGE_3_6, DEFAULT_AGE_RANGE, IMAGE_SUBJECT_PLACEHOLDER,
  PromptMessages, SAFETY_PROMPT, STYLE_WRAPPER,
};
use crate::i18n::{backend_messages, translate};
use crate::models::{ArticleDto, DomainDto};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn normalize_age_range(age_range: Option<&str>) -> &'static str {
  match age_range {
    Some(AGE_RANGE_3_6) => AGE_RANGE_3_6,
    Some(AGE_RANGE_10_14) => AGE_RANGE_10_14,
    _ => DEFAULT_AGE_RANGE,
  }
}

pub(crate) fn image_prompt(title: &str, locale: &str, age_range: &str) -> String {
  log::info!("Building image prompt for title='{title}'");
  let messages = backend_messages(locale);
  let age_instruction = age_instruction(&messages.prompts, normalize_age_range(Some(age_range)));

  format!("{}\n{age_instruction}", STYLE_WRAPPER.replace(IMAGE_SUBJECT_PLACEHOLDER, title))
}

/// Builds the child-safe text prompt sent to the optional LLM sidecar.
pub(crate) fn build_article_prompt(
  article: &ArticleDto,
  locale: &str,
  age_range: &str,
  curiosity_question: Option<&str>,
) -> String {
  log::info!("Building article prompt: article_id='{}', locale='{locale}'", article.id);
  let messages = backend_messages(locale);
  let prompts = messages.prompts;
  let age_instruction = age_instruction(&prompts, normalize_age_range(Some(age_range)));
  let language_instruction = prompts.article_instruction;
  let curiosity_focus = curiosity_question
    .map(|question| format!("\n{}", translate(&prompts.curiosity_focus, &[("question", question)])))
    .unwrap_or_default();

  format!(
    "{SAFETY_PROMPT}\n\n{language_instruction}\n{age_instruction}\n\n{}: {}\n{}: {}{}",
    prompts.article_subject_label,
    article.title,
    prompts.reliable_summary_label,
    article.summary,
    curiosity_focus
  )
}

pub(crate) fn build_questions_prompt(article: &ArticleDto, locale: &str, age_range: &str) -> String {
  log::info!("Building questions prompt: article_id='{}', locale='{locale}'", article.id);
  let messages = backend_messages(locale);
  let prompts = messages.prompts;
  let age_instruction = age_instruction(&prompts, normalize_age_range(Some(age_range)));
  let language_instruction = prompts.questions_instruction;

  format!(
    "{SAFETY_PROMPT}\n\n{language_instruction}\n{age_instruction}\n\n{}: {}\n{}: {}\n{}: {}",
    prompts.article_subject_label,
    article.title,
    prompts.reliable_summary_label,
    article.summary,
    prompts.safe_examples_label,
    article.questions.join(" | ")
  )
}

pub(crate) fn build_subcategories_prompt(domain: &DomainDto, locale: &str, age_range: &str) -> String {
  log::info!("Building subcategories prompt: domain_id='{}', locale='{locale}'", domain.id);
  let messages = backend_messages(locale);
  let prompts = messages.prompts;
  let count = domain.sections.len().to_string();
  let language_instruction = translate(&prompts.subcategories_instruction, &[("count", count.as_str())]);
  let age_instruction = age_instruction(&prompts, normalize_age_range(Some(age_range)));

  format!(
    "{SAFETY_PROMPT}\n\n{language_instruction}\n{age_instruction}\n\n{}: {}\n{}: {}\n{}: {}",
    prompts.category_label,
    domain.name,
    prompts.reliable_welcome_label,
    domain.welcome,
    prompts.existing_subcategories_label,
    domain.sections.iter().map(|section| section.name.as_str()).collect::<Vec<_>>().join(" | ")
  )
}

fn age_instruction(prompts: &PromptMessages, age_range: &str) -> &'static str {
  match age_range {
    AGE_RANGE_3_6 => prompts.age_instruction_3_6,
    AGE_RANGE_10_14 => prompts.age_instruction_10_14,
    _ => prompts.age_instruction_6_10,
  }
}

pub(crate) fn parse_generated_list(text: &str, expected_count: usize) -> Vec<String> {
  text
    .lines()
    .map(|line| clean_generated_list_item(line))
    .filter(|line| !line.is_empty())
    .take(expected_count)
    .collect()
}

fn clean_generated_list_item(line: &str) -> String {
  line
    .trim()
    .trim_start_matches(['-', '•', '*'])
    .trim_start()
    .trim_start_matches(|character: char| character.is_ascii_digit() || character == '.' || character == ')')
    .trim()
    .to_string()
}

#[cfg(test)]
mod tests {
  use super::*;

  fn sample_article() -> ArticleDto {
    ArticleDto {
      id: "arctic-fox".to_string(),
      title: "Renard polaire".to_string(),
      summary: "Son pelage l'aide à rester discret dans la neige.".to_string(),
      questions: vec![
        "Pourquoi change-t-il de couleur ?".to_string(),
        "Que mange-t-il ?".to_string(),
        "Où vit-il ?".to_string(),
      ],
    }
  }

  fn sample_domain() -> DomainDto {
    DomainDto {
      id: "nature".to_string(),
      name: "Nature".to_string(),
      icon: "Trees".to_string(),
      color: "from-emerald-400 to-teal-500".to_string(),
      welcome: "Explore les écosystèmes et les êtres vivants.".to_string(),
      sections: vec![
        crate::models::SectionDto { id: "animals".to_string(), name: "Animaux".to_string(), icon: "PawPrint".to_string(), article_id: "arctic-fox".to_string() },
        crate::models::SectionDto { id: "oceans".to_string(), name: "Océans".to_string(), icon: "Waves".to_string(), article_id: "coral-reef".to_string() },
      ],
    }
  }


  #[test]
  fn article_prompt_focuses_selected_curiosity_question() {
    let prompt = build_article_prompt(
      &sample_article(),
      "fr",
      "10-14",
      Some("Pourquoi change-t-il de couleur ?"),
    );

    assert!(prompt.contains("Question de curiosité choisie: Pourquoi change-t-il de couleur ?"));
    assert!(prompt.contains("si une question est indiquée"));
    assert!(prompt.contains("vocabulaire plus précis"));
    assert!(!prompt.contains("Questions proposées"));
  }

  #[test]
  fn article_prompt_does_not_request_follow_up_questions() {
    let prompt = build_article_prompt(&sample_article(), "fr", "6-10", None);

    assert!(!prompt.contains("termine par exactement trois questions"));
  }

  #[test]
  fn questions_prompt_is_separate_from_article_content_prompt() {
    let prompt = build_questions_prompt(&sample_article(), "fr", "3-6");

    assert!(prompt.contains("Écris exactement trois courtes questions"));
    assert!(prompt.contains("phrases très courtes"));
    assert!(prompt.contains("Exemples sûrs"));
    assert!(!prompt.contains("2 courts paragraphes"));
  }

  #[test]
  fn article_prompt_uses_english_backend_i18n_file() {
    let prompt = build_article_prompt(&sample_article(), "en", "6-10", Some("How does it stay warm?"));

    assert!(prompt.contains("Answer in English. Stay strictly focused"));
    assert!(prompt.contains("clear vocabulary"));
    assert!(prompt.contains("Selected curiosity question: How does it stay warm?"));
    assert!(prompt.contains("Reliable summary"));
  }

  #[test]
  fn subcategories_prompt_uses_domain_section_count_and_diversity_rules() {
    let prompt = build_subcategories_prompt(&sample_domain(), "fr", "10-14");

    assert!(prompt.contains("Suggère exactement 2 sous-thèmes sûrs"));
    assert!(prompt.contains("vocabulaire plus précis"));
    assert!(prompt.contains("Sous-catégories existantes: Animaux | Océans"));
  }

  #[test]
  fn image_prompt_stays_on_subject_and_child_safe() {
    let prompt = image_prompt("Renard polaire", "fr", "3-6");

    assert!(prompt.contains("Renard polaire only"));
    assert!(prompt.contains("child-safe educational illustration"));
    assert!(prompt.contains("phrases très courtes"));
    assert!(prompt.contains("no text"));
  }

  #[test]
  fn invalid_age_range_uses_default_profile() {
    assert_eq!(normalize_age_range(Some("12")), DEFAULT_AGE_RANGE);
    assert_eq!(normalize_age_range(None), DEFAULT_AGE_RANGE);
  }

  #[test]
  fn parses_generated_questions_as_three_clean_items() {
    let questions = parse_generated_list(
      "1. Pourquoi sa fourrure est-elle blanche ?\n- Que mange-t-il ?\n• Où dort-il ?\nQuatrième question ?",
      3,
    );

    assert_eq!(
      questions,
      vec![
        "Pourquoi sa fourrure est-elle blanche ?".to_string(),
        "Que mange-t-il ?".to_string(),
        "Où dort-il ?".to_string(),
      ]
    );
  }
}

pub(crate) fn now_millis() -> u128 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|duration| duration.as_millis())
    .unwrap_or_default()
}
