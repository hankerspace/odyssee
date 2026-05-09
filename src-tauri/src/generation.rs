//! Prompt construction and offline fallback content generation.

use crate::constants::{SAFETY_PROMPT, STYLE_WRAPPER};
use crate::models::ArticleDto;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn image_prompt(title: &str) -> String {
  log::info!("Building image prompt for title='{title}'");
  STYLE_WRAPPER.replace("[SUBJECT]", title)
}

/// Builds the child-safe text prompt sent to the optional LLM sidecar.
pub(crate) fn build_article_prompt(
  article: &ArticleDto,
  locale: &str,
  curiosity_question: Option<&str>,
) -> String {
  log::info!("Building article prompt: article_id='{}', locale='{locale}'", article.id);
  let language_instruction = if locale == "en" {
    "Answer in English for a child aged 6 to 10. Use short paragraphs and end with exactly three curiosity questions."
  } else {
    "Réponds en français pour un enfant de 6 à 10 ans. Utilise des paragraphes courts et termine par exactement trois questions de curiosité."
  };
  let curiosity_focus = curiosity_question
    .map(|question| {
      if locale == "en" {
        format!("\nSelected curiosity question: {question}\nFocus the explanation on this question.")
      } else {
        format!("\nQuestion de curiosité choisie: {question}\nCentre l'explication sur cette question.")
      }
    })
    .unwrap_or_default();

  format!(
    "{SAFETY_PROMPT}\n\n{language_instruction}\n\nSujet: {}\nRésumé fiable: {}\nQuestions proposées: {}{}",
    article.title,
    article.summary,
    article.questions.join(" | "),
    curiosity_focus
  )
}

pub(crate) fn fallback_article_text(
  article: &ArticleDto,
  locale: &str,
  curiosity_question: Option<&str>,
  sidecar_error: Option<&str>,
) -> String {
  log::info!("Rendering fallback article text: article_id='{}', locale='{locale}'", article.id);
  let intro = if locale == "en" {
    "Local demo explanation"
  } else {
    "Explication locale de démonstration"
  };
  let curiosity_label = if locale == "en" {
    "Curiosity paths"
  } else {
    "Pistes de curiosité"
  };
  let sidecar_note = sidecar_error
    .map(|error| format!("\n\nFallback mode active: {error}"))
    .unwrap_or_default();
  let curiosity_answer = curiosity_question
    .map(|question| {
      if locale == "en" {
        format!("\n\nCuriosity question: {question}\nShort answer: {}", article.summary)
      } else {
        format!("\n\nQuestion de curiosité: {question}\nRéponse courte: {}", article.summary)
      }
    })
    .unwrap_or_default();

  format!(
    "{intro}: {}\n\n{}{}\n\n{curiosity_label}:\n• {}\n• {}\n• {}{}",
    article.title,
    article.summary,
    curiosity_answer,
    article.questions.first().cloned().unwrap_or_default(),
    article.questions.get(1).cloned().unwrap_or_default(),
    article.questions.get(2).cloned().unwrap_or_default(),
    sidecar_note
  )
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

  #[test]
  fn article_prompt_focuses_selected_curiosity_question() {
    let prompt = build_article_prompt(
      &sample_article(),
      "fr",
      Some("Pourquoi change-t-il de couleur ?"),
    );

    assert!(prompt.contains("Question de curiosité choisie: Pourquoi change-t-il de couleur ?"));
    assert!(prompt.contains("Centre l'explication sur cette question."));
  }

  #[test]
  fn fallback_text_answers_selected_curiosity_question() {
    let text = fallback_article_text(
      &sample_article(),
      "fr",
      Some("Pourquoi change-t-il de couleur ?"),
      None,
    );

    assert!(text.contains("Question de curiosité: Pourquoi change-t-il de couleur ?"));
    assert!(text.contains("Réponse courte: Son pelage l'aide à rester discret dans la neige."));
  }
}

pub(crate) fn write_placeholder_svg(
  output_path: &Path,
  article: &ArticleDto,
  locale: &str,
) -> Result<(), String> {
  log::info!(
    "Writing fallback SVG illustration: article_id='{}', path='{}'",
    article.id,
    output_path.display()
  );
  let label = if locale == "en" {
    "offline illustration"
  } else {
    "illustration hors ligne"
  };
  let safe_title = escape_xml(&article.title);
  let safe_label = escape_xml(label);
  let svg = format!(
    r##"<svg xmlns="http://www.w3.org/2000/svg" width="960" height="640" viewBox="0 0 960 640">
  <defs>
    <linearGradient id="bg" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="#dbeafe"/>
      <stop offset="55%" stop-color="#f5d0fe"/>
      <stop offset="100%" stop-color="#dcfce7"/>
    </linearGradient>
  </defs>
  <rect width="960" height="640" rx="48" fill="url(#bg)"/>
  <circle cx="220" cy="170" r="80" fill="#ffffff" opacity="0.65"/>
  <circle cx="750" cy="450" r="120" fill="#ffffff" opacity="0.5"/>
  <rect x="170" y="190" width="620" height="260" rx="42" fill="#ffffff" opacity="0.82"/>
  <text x="480" y="305" text-anchor="middle" font-family="Arial, sans-serif" font-size="48" font-weight="700" fill="#1e293b">{safe_title}</text>
  <text x="480" y="372" text-anchor="middle" font-family="Arial, sans-serif" font-size="28" fill="#475569">{safe_label}</text>
  <text x="480" y="430" text-anchor="middle" font-family="Arial, sans-serif" font-size="20" fill="#64748b">stable-diffusion.cpp will run when the model is installed</text>
</svg>"##
  );

  fs::write(output_path, svg).map_err(|error| error.to_string())
}

pub(crate) fn now_millis() -> u128 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|duration| duration.as_millis())
    .unwrap_or_default()
}

fn escape_xml(value: &str) -> String {
  value
    .replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
    .replace('"', "&quot;")
    .replace('\'', "&apos;")
}