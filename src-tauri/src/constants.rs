//! Shared runtime constants for local inference, storage, and prompt safety.

/// Safety system instruction injected before every LLM request.
pub(crate) const SAFETY_PROMPT: &str = "You are a children encyclopedia. Never mention violent, political, or inappropriate content. Stay factual and kind.";

/// Visual style wrapper used for every stable-diffusion.cpp image prompt.
pub(crate) const STYLE_WRAPPER: &str = "A professional educational illustration of [SUBJECT], sticker style, clean lines, bright colors, white background, high quality for children encyclopedia.";

/// SQLite database file created inside the application data directory.
pub(crate) const DB_FILE_NAME: &str = "odyssee.sqlite";

/// Default text model expected in the local models directory.
pub(crate) const LLM_MODEL_FILE: &str = "phi-4-mini-instruct-q4_k_m.gguf";

/// Public GGUF download URL used to bootstrap the text model on first startup.
pub(crate) const LLM_MODEL_URL: &str = "https://huggingface.co/matrixportalx/Phi-4-mini-instruct-Q4_K_M-GGUF/resolve/main/phi-4-mini-instruct-q4_k_m.gguf";

/// Default image model expected in the local models directory.
pub(crate) const IMAGE_MODEL_FILE: &str = "flux-2-klein-base-4b-Q4_0.gguf";

/// Public GGUF download URL used to bootstrap the image model on first startup.
pub(crate) const IMAGE_MODEL_URL: &str = "https://huggingface.co/leejet/FLUX.2-klein-base-4B-GGUF/resolve/main/flux-2-klein-base-4b-Q4_0.gguf";