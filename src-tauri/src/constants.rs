//! Shared runtime constants for local inference, storage, and prompt safety.

/// Safety system instruction injected before every LLM request.
pub(crate) const SAFETY_PROMPT: &str = "You are an offline children encyclopedia assistant for Odyssée Kids. Answer only the requested topic or question. If a detail is uncertain, say so simply. Never introduce violent, political, sexual, scary, discriminatory, advertising, or inappropriate content. Stay factual, kind, curious, and educational.";

/// Visual style wrapper used for every stable-diffusion.cpp image prompt.
pub(crate) const STYLE_WRAPPER: &str = "A bright child-safe educational illustration of [SUBJECT] only, simple encyclopedia sticker style, clean lines, friendly colors, white background, no text, no logos, no people in danger, high quality.";

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