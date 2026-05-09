//! Shared runtime constants for local inference, storage, and prompt safety.

/// Backend i18n file used for French user-facing messages outside LLM prompts.
pub(crate) const FR_BACKEND_MESSAGES: &str = include_str!("../i18n/fr.json");

/// Backend i18n file used for English user-facing messages outside LLM prompts.
pub(crate) const EN_BACKEND_MESSAGES: &str = include_str!("../i18n/en.json");

/// Safety system instruction injected before every LLM request.
pub(crate) const SAFETY_PROMPT: &str = "You are the offline children encyclopedia assistant for Odyssée Kids. Treat the SYSTEM PROMPT as rules and the USER PROMPT as the only task to answer. Answer only the requested topic or question. If a detail is uncertain, say so simply. Never introduce violent, political, sexual, scary, discriminatory, advertising, or inappropriate content. Stay factual, kind, curious, and educational.";

/// Visual style wrapper used for every stable-diffusion.cpp image prompt.
pub(crate) const STYLE_WRAPPER: &str = "Child-safe educational encyclopedia illustration. Subject: [SUBJECT]. Show only this subject, centered and instantly recognizable, with accurate simple features. Bright friendly colors, clean rounded shapes, soft natural light, uncluttered white background, sticker icon style. no text, no letters, no logos, no watermark, no extra characters, no scary or dangerous scene. High quality.";

/// Placeholder replaced by the article title inside the image prompt wrapper.
pub(crate) const IMAGE_SUBJECT_PLACEHOLDER: &str = "[SUBJECT]";

/// Age range used when no explicit range is provided by the frontend.
pub(crate) const DEFAULT_AGE_RANGE: &str = AGE_RANGE_6_10;

/// Preschool audience age range.
pub(crate) const AGE_RANGE_3_6: &str = "3-6";

/// Primary-school audience age range.
pub(crate) const AGE_RANGE_6_10: &str = "6-10";

/// Early-teen audience age range.
pub(crate) const AGE_RANGE_10_14: &str = "10-14";

/// Localized LLM prompt snippets used by prompt builders.
#[derive(Clone, Copy)]
pub(crate) struct PromptMessages {
    pub(crate) article_instruction: &'static str,
    pub(crate) curiosity_focus: &'static str,
    pub(crate) questions_instruction: &'static str,
    pub(crate) subcategories_instruction: &'static str,
    pub(crate) age_instruction_3_6: &'static str,
    pub(crate) age_instruction_6_10: &'static str,
    pub(crate) age_instruction_10_14: &'static str,
    pub(crate) article_subject_label: &'static str,
    pub(crate) reliable_summary_label: &'static str,
    pub(crate) safe_examples_label: &'static str,
    pub(crate) category_label: &'static str,
    pub(crate) reliable_welcome_label: &'static str,
    pub(crate) existing_subcategories_label: &'static str,
}

/// French instruction for article generation.
pub(crate) const FR_LLM_ARTICLE_PROMPT: &str = "Réponds uniquement en français. Produis une réponse d'encyclopédie pour enfant, claire et chaleureuse. Si le USER PROMPT contient une question principale, réponds précisément à cette question dès la première phrase utile, puis ajoute seulement les explications nécessaires pour la comprendre. Ne rédige pas une fiche générale avant d'avoir traité la question. Utilise uniquement le résumé fiable et des connaissances générales sûres. Varie les exemples et l'analogie pour éviter les réponses répétitives. Structure: titre court, 3 paragraphes courts, une analogie concrète. N'ajoute pas de questions à la fin.";

/// French instruction added when a child asks a focused question.
pub(crate) const FR_LLM_CURIOSITY_FOCUS_PROMPT: &str = "Question principale de l'enfant: {question}\nTâche prioritaire: réponds spécifiquement à cette question, sans la remplacer par une présentation générale du sujet.";

/// French instruction for follow-up question generation.
pub(crate) const FR_LLM_QUESTIONS_PROMPT: &str = "Écris exactement trois courtes questions de curiosité en français. Chaque question doit être différente des exemples, concrète, directement liée au sujet précis et au résumé fiable, et sûre pour un enfant. Varie les angles: fonctionnement, comparaison, protection ou observation. Retourne une question par ligne et aucune réponse.";

/// French instruction for subcategory generation.
pub(crate) const FR_LLM_SUBCATEGORIES_PROMPT: &str = "Suggère exactement {count} sous-thèmes sûrs en français pour cette catégorie d'encyclopédie. Chaque libellé doit être distinct, concret, centré sur le domaine et assez large pour regrouper plusieurs sujets. Évite les synonymes, doublons et termes vagues comme Découvertes ou Général. Retourne un court libellé par ligne et aucune explication.";

/// French prompt guidance for ages 3 to 6.
pub(crate) const FR_LLM_AGE_PROMPT_3_6: &str = "Adapte tout le contenu à un enfant de 3 à 6 ans : phrases très courtes, vocabulaire très simple, exemples du quotidien, ton doux et rassurant.";

/// French prompt guidance for ages 6 to 10.
pub(crate) const FR_LLM_AGE_PROMPT_6_10: &str = "Adapte tout le contenu à un enfant de 6 à 10 ans : vocabulaire clair, explications concrètes, analogies simples et détails progressifs.";

/// French prompt guidance for ages 10 to 14.
pub(crate) const FR_LLM_AGE_PROMPT_10_14: &str = "Adapte tout le contenu à un enfant de 10 à 14 ans : vocabulaire plus précis, liens de cause à effet, nuances utiles et défi sûr.";

/// French article subject label inserted in prompts.
pub(crate) const FR_LLM_ARTICLE_SUBJECT_LABEL: &str = "Sujet";

/// French reliable summary label inserted in prompts.
pub(crate) const FR_LLM_RELIABLE_SUMMARY_LABEL: &str = "Résumé fiable";

/// French safe examples label inserted in prompts.
pub(crate) const FR_LLM_SAFE_EXAMPLES_LABEL: &str = "Exemples sûrs";

/// French category label inserted in prompts.
pub(crate) const FR_LLM_CATEGORY_LABEL: &str = "Catégorie";

/// French welcome/context label inserted in prompts.
pub(crate) const FR_LLM_RELIABLE_WELCOME_LABEL: &str = "Accueil fiable";

/// French existing subcategories label inserted in prompts.
pub(crate) const FR_LLM_EXISTING_SUBCATEGORIES_LABEL: &str = "Sous-catégories existantes";

/// English instruction for article generation.
pub(crate) const EN_LLM_ARTICLE_PROMPT: &str = "Answer only in English. Produce a clear, warm encyclopedia answer for a child. If the USER PROMPT contains a main question, answer that exact question in the first useful sentence, then add only the explanation needed to understand it. Do not write a general article before addressing the question. Use only the reliable summary and safe general knowledge. Vary examples and the analogy to avoid repetitive answers. Structure: short title, 3 short paragraphs, one concrete analogy. Do not add follow-up questions.";

/// English instruction added when a child asks a focused question.
pub(crate) const EN_LLM_CURIOSITY_FOCUS_PROMPT: &str = "Child's main question: {question}\nPriority task: answer this question specifically, without replacing it with a general presentation of the subject.";

/// English instruction for follow-up question generation.
pub(crate) const EN_LLM_QUESTIONS_PROMPT: &str = "Write exactly three short curiosity questions in English. Each question must be different from the examples, concrete, directly linked to the precise subject and reliable summary, and safe for a child. Vary the angles: how it works, comparison, protection, or observation. Return one question per line and no answers.";

/// English instruction for subcategory generation.
pub(crate) const EN_LLM_SUBCATEGORIES_PROMPT: &str = "Suggest exactly {count} child-safe subthemes in English for this encyclopedia category. Each label must be distinct, concrete, centered on the domain, and broad enough to group several subjects. Avoid synonyms, duplicates, and vague terms like Discoveries or General. Return one short label per line and no explanations.";

/// English prompt guidance for ages 3 to 6.
pub(crate) const EN_LLM_AGE_PROMPT_3_6: &str = "Adapt all content for a child aged 3 to 6: very short sentences, very simple words, everyday examples, and a gentle reassuring tone.";

/// English prompt guidance for ages 6 to 10.
pub(crate) const EN_LLM_AGE_PROMPT_6_10: &str = "Adapt all content for a child aged 6 to 10: clear vocabulary, concrete explanations, simple analogies, and progressive details.";

/// English prompt guidance for ages 10 to 14.
pub(crate) const EN_LLM_AGE_PROMPT_10_14: &str = "Adapt all content for a child aged 10 to 14: more precise vocabulary, cause-and-effect links, useful nuance, and a safe challenge.";

/// English article subject label inserted in prompts.
pub(crate) const EN_LLM_ARTICLE_SUBJECT_LABEL: &str = "Subject";

/// English reliable summary label inserted in prompts.
pub(crate) const EN_LLM_RELIABLE_SUMMARY_LABEL: &str = "Reliable summary";

/// English safe examples label inserted in prompts.
pub(crate) const EN_LLM_SAFE_EXAMPLES_LABEL: &str = "Safe examples";

/// English category label inserted in prompts.
pub(crate) const EN_LLM_CATEGORY_LABEL: &str = "Category";

/// English welcome/context label inserted in prompts.
pub(crate) const EN_LLM_RELIABLE_WELCOME_LABEL: &str = "Reliable welcome";

/// English existing subcategories label inserted in prompts.
pub(crate) const EN_LLM_EXISTING_SUBCATEGORIES_LABEL: &str = "Existing subcategories";

/// French LLM prompts used by prompt builders.
pub(crate) const FR_PROMPT_MESSAGES: PromptMessages = PromptMessages {
    article_instruction: FR_LLM_ARTICLE_PROMPT,
    curiosity_focus: FR_LLM_CURIOSITY_FOCUS_PROMPT,
    questions_instruction: FR_LLM_QUESTIONS_PROMPT,
    subcategories_instruction: FR_LLM_SUBCATEGORIES_PROMPT,
    age_instruction_3_6: FR_LLM_AGE_PROMPT_3_6,
    age_instruction_6_10: FR_LLM_AGE_PROMPT_6_10,
    age_instruction_10_14: FR_LLM_AGE_PROMPT_10_14,
    article_subject_label: FR_LLM_ARTICLE_SUBJECT_LABEL,
    reliable_summary_label: FR_LLM_RELIABLE_SUMMARY_LABEL,
    safe_examples_label: FR_LLM_SAFE_EXAMPLES_LABEL,
    category_label: FR_LLM_CATEGORY_LABEL,
    reliable_welcome_label: FR_LLM_RELIABLE_WELCOME_LABEL,
    existing_subcategories_label: FR_LLM_EXISTING_SUBCATEGORIES_LABEL,
};

/// English LLM prompts used by prompt builders.
pub(crate) const EN_PROMPT_MESSAGES: PromptMessages = PromptMessages {
    article_instruction: EN_LLM_ARTICLE_PROMPT,
    curiosity_focus: EN_LLM_CURIOSITY_FOCUS_PROMPT,
    questions_instruction: EN_LLM_QUESTIONS_PROMPT,
    subcategories_instruction: EN_LLM_SUBCATEGORIES_PROMPT,
    age_instruction_3_6: EN_LLM_AGE_PROMPT_3_6,
    age_instruction_6_10: EN_LLM_AGE_PROMPT_6_10,
    age_instruction_10_14: EN_LLM_AGE_PROMPT_10_14,
    article_subject_label: EN_LLM_ARTICLE_SUBJECT_LABEL,
    reliable_summary_label: EN_LLM_RELIABLE_SUMMARY_LABEL,
    safe_examples_label: EN_LLM_SAFE_EXAMPLES_LABEL,
    category_label: EN_LLM_CATEGORY_LABEL,
    reliable_welcome_label: EN_LLM_RELIABLE_WELCOME_LABEL,
    existing_subcategories_label: EN_LLM_EXISTING_SUBCATEGORIES_LABEL,
};

/// SQLite database file created inside the application data directory.
pub(crate) const DB_FILE_NAME: &str = "odyssee.sqlite";

/// Default text model expected in the local models directory.
pub(crate) const LLM_MODEL_FILE: &str = "phi-4-mini-instruct-q4_k_m.gguf";

/// Public GGUF download URL used to bootstrap the text model on first startup.
pub(crate) const LLM_MODEL_URL: &str = "https://huggingface.co/matrixportalx/Phi-4-mini-instruct-Q4_K_M-GGUF/resolve/main/phi-4-mini-instruct-q4_k_m.gguf";

/// Default image model expected in the local models directory.
pub(crate) const IMAGE_MODEL_FILE: &str = "sd_turbo.safetensors";

/// Public single-file Stable Diffusion download URL used to bootstrap the image model on first startup.
pub(crate) const IMAGE_MODEL_URL: &str = "https://huggingface.co/stabilityai/sd-turbo/resolve/main/sd_turbo.safetensors";

/// User-facing error for the unsupported multi-file FLUX.2 image model.
pub(crate) const UNSUPPORTED_FLUX2_IMAGE_MODEL_ERROR: &str = "FLUX.2 Klein requires separate VAE and LLM companion models; \
the current image runner only supports single-file stable-diffusion.cpp models.";

/// Unsupported image model file kept for migration/status checks.
pub(crate) const UNSUPPORTED_FLUX2_IMAGE_MODEL_FILE: &str = "flux-2-klein-base-4b-Q4_0.gguf";

/// PNG magic bytes used to validate generated image files.
pub(crate) const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n'];