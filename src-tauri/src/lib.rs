mod commands;
mod constants;
mod generation;
mod hardware;
mod model_download;
mod models;
mod paths;
mod seeds;
mod sidecars;
mod storage;

#[cfg(any())]
#[allow(dead_code)]
mod legacy {
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

const SAFETY_PROMPT: &str = "You are a children encyclopedia. Never mention violent, political, or inappropriate content. Stay factual and kind.";
const STYLE_WRAPPER: &str = "A professional educational illustration of [SUBJECT], sticker style, clean lines, bright colors, white background, high quality for children encyclopedia.";
const DB_FILE_NAME: &str = "odyssee.sqlite";
const LLM_MODEL_FILE: &str = "phi-4-mini-instruct-q4_k_m.gguf";
const IMAGE_MODEL_FILE: &str = "sd_turbo.safetensors";

#[derive(Clone, Copy)]
struct DomainSeed {
  id: &'static str,
  icon: &'static str,
  color: &'static str,
  name_fr: &'static str,
  name_en: &'static str,
  welcome_fr: &'static str,
  welcome_en: &'static str,
}

#[derive(Clone, Copy)]
struct SectionSeed {
  id: &'static str,
  domain_id: &'static str,
  article_id: &'static str,
  icon: &'static str,
  name_fr: &'static str,
  name_en: &'static str,
}

#[derive(Clone, Copy)]
struct ArticleSeed {
  id: &'static str,
  title_fr: &'static str,
  title_en: &'static str,
  summary_fr: &'static str,
  summary_en: &'static str,
  questions_fr: [&'static str; 3],
  questions_en: [&'static str; 3],
}

const DOMAINS: [DomainSeed; 6] = [
  DomainSeed {
    id: "nature",
    icon: "Trees",
    color: "from-emerald-400 to-teal-500",
    name_fr: "Nature",
    name_en: "Nature",
    welcome_fr: "Bienvenue explorateur ! La nature cache des écosystèmes fascinants et des formes de vie surprenantes.",
    welcome_en: "Welcome explorer! Nature hides fascinating ecosystems and surprising living things.",
  },
  DomainSeed {
    id: "space",
    icon: "Rocket",
    color: "from-indigo-500 to-sky-500",
    name_fr: "Espace",
    name_en: "Space",
    welcome_fr: "Bienvenue astronaute ! Prépare-toi à voyager parmi les planètes, les étoiles et les missions spatiales.",
    welcome_en: "Welcome astronaut! Get ready to travel among planets, stars, and space missions.",
  },
  DomainSeed {
    id: "history",
    icon: "Landmark",
    color: "from-amber-400 to-orange-500",
    name_fr: "Histoire",
    name_en: "History",
    welcome_fr: "Bienvenue voyageur du temps ! Découvrons comment les humains ont inventé, exploré et transmis leurs idées.",
    welcome_en: "Welcome time traveler! Let us discover how people invented, explored, and shared ideas.",
  },
  DomainSeed {
    id: "technology",
    icon: "Cpu",
    color: "from-cyan-400 to-blue-500",
    name_fr: "Technologie",
    name_en: "Technology",
    welcome_fr: "Bienvenue constructeur ! Les machines et l'énergie propre aident à résoudre de vrais problèmes.",
    welcome_en: "Welcome builder! Machines and clean energy help solve real problems.",
  },
  DomainSeed {
    id: "human-body",
    icon: "HeartPulse",
    color: "from-rose-400 to-pink-500",
    name_fr: "Corps humain",
    name_en: "Human Body",
    welcome_fr: "Bienvenue scientifique ! Ton corps est une merveille faite d'os, de sens et de milliards de messages.",
    welcome_en: "Welcome scientist! Your body is a wonder made of bones, senses, and billions of messages.",
  },
  DomainSeed {
    id: "arts",
    icon: "Palette",
    color: "from-purple-400 to-fuchsia-500",
    name_fr: "Arts",
    name_en: "Arts",
    welcome_fr: "Bienvenue créateur ! Les sons, les couleurs et les formes racontent des histoires sans écran.",
    welcome_en: "Welcome creator! Sounds, colors, and shapes tell stories without screens.",
  },
];

const SECTIONS: [SectionSeed; 18] = [
  SectionSeed { id: "animals", domain_id: "nature", article_id: "arctic-fox", icon: "PawPrint", name_fr: "Animaux", name_en: "Animals" },
  SectionSeed { id: "oceans", domain_id: "nature", article_id: "coral-reef", icon: "Waves", name_fr: "Océans", name_en: "Oceans" },
  SectionSeed { id: "forests", domain_id: "nature", article_id: "redwood", icon: "TreePine", name_fr: "Forêts", name_en: "Forests" },
  SectionSeed { id: "planets", domain_id: "space", article_id: "saturn", icon: "Orbit", name_fr: "Planètes", name_en: "Planets" },
  SectionSeed { id: "missions", domain_id: "space", article_id: "apollo-11", icon: "Moon", name_fr: "Missions", name_en: "Missions" },
  SectionSeed { id: "stars", domain_id: "space", article_id: "sun", icon: "Sun", name_fr: "Étoiles", name_en: "Stars" },
  SectionSeed { id: "civilizations", domain_id: "history", article_id: "egypt", icon: "Landmark", name_fr: "Civilisations", name_en: "Civilizations" },
  SectionSeed { id: "inventions", domain_id: "history", article_id: "printing-press", icon: "BookOpen", name_fr: "Inventions", name_en: "Inventions" },
  SectionSeed { id: "explorers", domain_id: "history", article_id: "magellan", icon: "Compass", name_fr: "Explorateurs", name_en: "Explorers" },
  SectionSeed { id: "robots", domain_id: "technology", article_id: "mars-rover", icon: "Bot", name_fr: "Robots", name_en: "Robots" },
  SectionSeed { id: "energy", domain_id: "technology", article_id: "solar-panels", icon: "Zap", name_fr: "Énergie", name_en: "Energy" },
  SectionSeed { id: "transport", domain_id: "technology", article_id: "high-speed-train", icon: "Train", name_fr: "Transport", name_en: "Transport" },
  SectionSeed { id: "skeleton", domain_id: "human-body", article_id: "femur", icon: "Bone", name_fr: "Squelette", name_en: "Skeleton" },
  SectionSeed { id: "brain", domain_id: "human-body", article_id: "neurons", icon: "Brain", name_fr: "Cerveau", name_en: "Brain" },
  SectionSeed { id: "senses", domain_id: "human-body", article_id: "eyes", icon: "Eye", name_fr: "Sens", name_en: "Senses" },
  SectionSeed { id: "music", domain_id: "arts", article_id: "violin", icon: "Music", name_fr: "Musique", name_en: "Music" },
  SectionSeed { id: "painting", domain_id: "arts", article_id: "watercolor", icon: "Brush", name_fr: "Peinture", name_en: "Painting" },
  SectionSeed { id: "sculpture", domain_id: "arts", article_id: "marble", icon: "Gem", name_fr: "Sculpture", name_en: "Sculpture" },
];

const ARTICLES: [ArticleSeed; 18] = [
  ArticleSeed { id: "arctic-fox", title_fr: "Renard polaire", title_en: "Arctic Fox", summary_fr: "Le renard polaire vit dans la toundra froide. Sa fourrure épaisse, ses petites oreilles et ses pattes poilues l'aident à garder la chaleur. Il change parfois de couleur selon la saison pour mieux se cacher.", summary_en: "The arctic fox lives in cold tundra. Thick fur, small ears, and furry paws help it keep heat. Its coat can change color with the seasons for camouflage.", questions_fr: ["Comment reste-t-il au chaud en hiver ?", "Pourquoi son pelage change-t-il ?", "Que mange-t-il dans la toundra ?"], questions_en: ["How does it stay warm in winter?", "Why does its coat change color?", "What does it eat in the tundra?"] },
  ArticleSeed { id: "coral-reef", title_fr: "Récif corallien", title_en: "Coral Reef", summary_fr: "Un récif corallien ressemble à une ville sous-marine. De minuscules animaux appelés coraux construisent des abris où poissons, crustacés et plantes marines vivent ensemble.", summary_en: "A coral reef is like an underwater city. Tiny animals called corals build shelters where fish, crustaceans, and sea plants live together.", questions_fr: ["Pourquoi les récifs sont-ils colorés ?", "Comment les coraux construisent-ils ?", "Comment peut-on les protéger ?"], questions_en: ["Why are reefs colorful?", "How do corals build reefs?", "How can we protect them?"] },
  ArticleSeed { id: "redwood", title_fr: "Séquoia géant", title_en: "Redwood Tree", summary_fr: "Les séquoias font partie des arbres les plus hauts du monde. Leur écorce épaisse les protège et leurs forêts abritent de nombreux animaux.", summary_en: "Redwoods are among the tallest trees on Earth. Thick bark protects them, and their forests shelter many animals.", questions_fr: ["Jusqu'à quelle hauteur peuvent-ils pousser ?", "Pourquoi leur écorce est-elle épaisse ?", "Où vivent les séquoias ?"], questions_en: ["How tall can they grow?", "Why is their bark thick?", "Where do redwoods live?"] },
  ArticleSeed { id: "saturn", title_fr: "Saturne", title_en: "Saturn", summary_fr: "Saturne est une planète géante entourée d'anneaux brillants faits de glace et de roches. Elle est si grande que de nombreuses lunes tournent autour d'elle.", summary_en: "Saturn is a giant planet surrounded by bright rings made of ice and rock. It is so large that many moons orbit it.", questions_fr: ["De quoi sont faits les anneaux ?", "Combien de lunes a Saturne ?", "Peut-on marcher sur Saturne ?"], questions_en: ["What are the rings made of?", "How many moons does Saturn have?", "Could we stand on Saturn?"] },
  ArticleSeed { id: "apollo-11", title_fr: "Apollo 11", title_en: "Apollo 11", summary_fr: "Apollo 11 est la mission qui a permis aux premiers humains de marcher sur la Lune en 1969. Les astronautes ont étudié le sol lunaire avant de rentrer sur Terre.", summary_en: "Apollo 11 was the mission that let the first humans walk on the Moon in 1969. Astronauts studied lunar soil before returning to Earth.", questions_fr: ["Qui a marché le premier sur la Lune ?", "Comment sont-ils rentrés ?", "Quels outils ont-ils utilisés ?"], questions_en: ["Who walked on the Moon first?", "How did they return home?", "What tools did they use?"] },
  ArticleSeed { id: "sun", title_fr: "Soleil", title_en: "The Sun", summary_fr: "Le Soleil est une étoile. Il donne lumière et chaleur à la Terre, ce qui permet aux plantes de pousser et à la vie d'exister.", summary_en: "The Sun is a star. It gives Earth light and heat, helping plants grow and life exist.", questions_fr: ["Quelle est sa température ?", "Pourquoi paraît-il jaune ?", "Qu'est-ce que le vent solaire ?"], questions_en: ["How hot is it?", "Why does it look yellow?", "What is solar wind?"] },
  ArticleSeed { id: "egypt", title_fr: "Égypte antique", title_en: "Ancient Egypt", summary_fr: "L'Égypte antique s'est développée près du Nil. Ses habitants ont construit des pyramides, inventé des écritures et étudié les étoiles.", summary_en: "Ancient Egypt grew near the Nile. Its people built pyramids, invented writing systems, and studied the stars.", questions_fr: ["Pourquoi le Nil était-il important ?", "Comment construisait-on les pyramides ?", "Que sont les hiéroglyphes ?"], questions_en: ["Why was the Nile important?", "How were pyramids built?", "What are hieroglyphs?"] },
  ArticleSeed { id: "printing-press", title_fr: "Imprimerie", title_en: "Printing Press", summary_fr: "L'imprimerie a permis de fabriquer des livres plus vite. Les idées, les histoires et les découvertes ont ainsi circulé auprès de beaucoup plus de personnes.", summary_en: "The printing press made books faster to produce. Ideas, stories, and discoveries could reach many more people.", questions_fr: ["Qui a amélioré l'imprimerie ?", "Pourquoi les livres coûtaient-ils moins cher ?", "Comment a-t-elle changé l'école ?"], questions_en: ["Who improved the press?", "Why did books become cheaper?", "How did it change schools?"] },
  ArticleSeed { id: "magellan", title_fr: "Expédition de Magellan", title_en: "Magellan Expedition", summary_fr: "L'expédition de Magellan a réalisé le premier tour du monde en bateau. Le voyage a montré qu'on pouvait faire le tour de la Terre par la mer.", summary_en: "Magellan's expedition completed the first trip around the world by ship. The journey showed Earth could be circled by sea.", questions_fr: ["Pourquoi le voyage était-il difficile ?", "Combien de temps a-t-il duré ?", "Qu'ont appris les marins ?"], questions_en: ["Why was the journey difficult?", "How long did it last?", "What did sailors learn?"] },
  ArticleSeed { id: "mars-rover", title_fr: "Robot martien", title_en: "Mars Rover", summary_fr: "Un robot martien roule sur Mars pour étudier les roches, la météo et les traces d'eau ancienne. Il envoie ses découvertes aux scientifiques sur Terre.", summary_en: "A Mars rover drives on Mars to study rocks, weather, and signs of ancient water. It sends discoveries to scientists on Earth.", questions_fr: ["Comment se déplace-t-il ?", "Quelles données envoie-t-il ?", "Comment reçoit-il de l'énergie ?"], questions_en: ["How does it move?", "What data does it send?", "How is it powered?"] },
  ArticleSeed { id: "solar-panels", title_fr: "Panneaux solaires", title_en: "Solar Panels", summary_fr: "Les panneaux solaires transforment la lumière du Soleil en électricité. Ils peuvent alimenter des maisons, des écoles ou de petits appareils.", summary_en: "Solar panels turn sunlight into electricity. They can power homes, schools, or small devices.", questions_fr: ["Comment créent-ils de l'électricité ?", "Fonctionnent-ils par temps nuageux ?", "Pourquoi sont-ils utiles pour la planète ?"], questions_en: ["How do they make electricity?", "Do they work on cloudy days?", "Why are they good for the planet?"] },
  ArticleSeed { id: "high-speed-train", title_fr: "Train à grande vitesse", title_en: "High-Speed Train", summary_fr: "Un train à grande vitesse utilise des moteurs électriques puissants et des rails très réguliers pour transporter beaucoup de personnes rapidement.", summary_en: "A high-speed train uses powerful electric motors and very smooth tracks to carry many people quickly.", questions_fr: ["À quelle vitesse roule-t-il ?", "Pourquoi les rails sont-ils spéciaux ?", "Comment reste-t-il sûr ?"], questions_en: ["How fast can it go?", "Why are tracks special?", "How does it stay safe?"] },
  ArticleSeed { id: "femur", title_fr: "Fémur", title_en: "Femur", summary_fr: "Le fémur est l'os le plus long et l'un des plus solides du corps. Il relie la hanche au genou et aide à marcher, courir et sauter.", summary_en: "The femur is the longest and one of the strongest bones in the body. It connects hip to knee and helps you walk, run, and jump.", questions_fr: ["Pourquoi est-il si solide ?", "Comment rejoint-il la hanche ?", "Comment garder des os en bonne santé ?"], questions_en: ["Why is it so strong?", "How does it connect to the hip?", "How can we keep bones healthy?"] },
  ArticleSeed { id: "neurons", title_fr: "Neurones", title_en: "Neurons", summary_fr: "Les neurones sont de minuscules cellules qui transportent des messages dans le cerveau et le corps. Ils t'aident à bouger, apprendre et ressentir.", summary_en: "Neurons are tiny cells that carry messages through the brain and body. They help you move, learn, and feel.", questions_fr: ["Comment envoient-ils des signaux ?", "Pourquoi le sommeil les aide-t-il ?", "Comment apprendre les change-t-il ?"], questions_en: ["How do they send signals?", "Why does sleep help them?", "How does learning change them?"] },
  ArticleSeed { id: "eyes", title_fr: "Yeux", title_en: "Eyes", summary_fr: "Les yeux captent la lumière et envoient des signaux au cerveau. Le cerveau transforme ces signaux en images que tu peux comprendre.", summary_en: "Eyes collect light and send signals to the brain. The brain turns those signals into pictures you can understand.", questions_fr: ["Comment fonctionnent les pupilles ?", "Pourquoi cligne-t-on des yeux ?", "Comment protéger sa vue ?"], questions_en: ["How do pupils work?", "Why do we blink?", "How can we protect eyesight?"] },
  ArticleSeed { id: "violin", title_fr: "Violon", title_en: "Violin", summary_fr: "Le violon produit un son quand ses cordes vibrent. L'archet frotte les cordes et le corps en bois rend le son plus fort et plus riche.", summary_en: "A violin makes sound when its strings vibrate. The bow rubs the strings, and the wooden body makes the sound louder and richer.", questions_fr: ["Pourquoi l'archet crée-t-il un son ?", "Combien de cordes possède-t-il ?", "Qu'est-ce qu'un orchestre ?"], questions_en: ["Why does the bow make sound?", "How many strings does it have?", "What is an orchestra?"] },
  ArticleSeed { id: "watercolor", title_fr: "Aquarelle", title_en: "Watercolor", summary_fr: "L'aquarelle utilise de l'eau et des pigments pour créer des couleurs transparentes. Les artistes superposent les couches pour obtenir de la lumière et des nuances.", summary_en: "Watercolor uses water and pigments to create transparent colors. Artists layer washes to make light and shades.", questions_fr: ["Comment mélange-t-on les couleurs ?", "Pourquoi le papier compte-t-il ?", "Qu'est-ce qu'un lavis ?"], questions_en: ["How do artists blend colors?", "Why does paper matter?", "What is a wash?"] },
  ArticleSeed { id: "marble", title_fr: "Sculpture en marbre", title_en: "Marble Sculpture", summary_fr: "Une sculpture en marbre naît quand un artiste retire peu à peu de la pierre avec des outils. Chaque geste révèle une forme cachée.", summary_en: "A marble sculpture is made as an artist slowly removes stone with tools. Each movement reveals a hidden shape.", questions_fr: ["Quels outils utilise-t-on ?", "Pourquoi choisir le marbre ?", "Combien de temps peut-elle durer ?"], questions_en: ["What tools are used?", "Why choose marble?", "How long can it last?"] },
];

#[derive(Serialize)]
struct RuntimeProfile {
  hardware: String,
  accelerator: String,
  expected_performance: String,
  llm_flags: Vec<String>,
  image_flags: Vec<String>,
  model_directory: String,
  cache_directory: String,
  safety_prompt: String,
  style_wrapper: String,
  llm_binary: String,
  image_binary: String,
  llm_model: String,
  image_model: String,
  database_path: String,
}

#[derive(Serialize)]
struct HardwareProfile {
  hardware: String,
  accelerator: String,
  expected_performance: String,
}

#[derive(Deserialize)]
struct ArticleRequest {
  article_id: String,
  locale: Option<String>,
}

#[derive(Serialize)]
struct CatalogResponse {
  domains: Vec<DomainDto>,
}

#[derive(Serialize)]
struct DomainDto {
  id: String,
  name: String,
  icon: String,
  color: String,
  welcome: String,
  sections: Vec<SectionDto>,
}

#[derive(Serialize)]
struct SectionDto {
  id: String,
  name: String,
  icon: String,
  article_id: String,
}

#[derive(Serialize)]
struct ArticleDto {
  id: String,
  title: String,
  summary: String,
  questions: Vec<String>,
}

#[derive(Serialize)]
struct GeneratedArticle {
  article: ArticleDto,
  generated_text: String,
  source: String,
  prompt: String,
  cached: bool,
}

#[derive(Serialize)]
struct GeneratedImage {
  article_id: String,
  prompt: String,
  source: String,
  image_path: String,
  cached: bool,
}

#[derive(Serialize)]
struct ModelStatus {
  model_directory: String,
  cache_directory: String,
  llm_binary: String,
  image_binary: String,
  llm_model: String,
  image_model: String,
  llm_ready: bool,
  image_ready: bool,
  database_path: String,
}

#[tauri::command]
fn detect_hardware() -> HardwareProfile {
  if cfg!(target_os = "macos") {
    return HardwareProfile {
      hardware: "Mac (Apple Silicon)".to_string(),
      accelerator: "Metal".to_string(),
      expected_performance: "Very smooth".to_string(),
    };
  }

  if has_command("nvidia-smi") {
    return HardwareProfile {
      hardware: "NVIDIA RTX".to_string(),
      accelerator: "CUDA".to_string(),
      expected_performance: "Near-instant inference".to_string(),
    };
  }

  if env::var("VULKAN_SDK").is_ok() || env::var("VK_ICD_FILENAMES").is_ok() {
    return HardwareProfile {
      hardware: "AMD / Intel GPU".to_string(),
      accelerator: "Vulkan".to_string(),
      expected_performance: "Good fluidity".to_string(),
    };
  }

  HardwareProfile {
    hardware: "PC without dedicated GPU".to_string(),
    accelerator: "CPU (AVX2/AVX512 when available)".to_string(),
    expected_performance: "Text is smooth, images in 15-45 seconds".to_string(),
  }
}

#[tauri::command]
fn get_runtime_profile() -> RuntimeProfile {
  let hardware_profile = detect_hardware();
  let model_directory = resolve_model_directory();
  let cache_directory = resolve_cache_directory();
  let paths = RuntimePaths::resolve();

  let (llm_flags, image_flags) = match hardware_profile.accelerator.as_str() {
    "CUDA" => (
      vec!["--ctx-size".into(), "4096".into(), "--n-gpu-layers".into(), "99".into()],
      vec!["--steps".into(), "4".into(), "--backend".into(), "cuda".into()],
    ),
    "Metal" => (
      vec!["--ctx-size".into(), "4096".into(), "--metal".into()],
      vec!["--steps".into(), "4".into(), "--backend".into(), "metal".into()],
    ),
    "Vulkan" => (
      vec!["--ctx-size".into(), "4096".into(), "--backend".into(), "vulkan".into()],
      vec!["--steps".into(), "4".into(), "--backend".into(), "vulkan".into()],
    ),
    _ => (
      vec!["--ctx-size".into(), "4096".into(), "--threads".into(), "4".into()],
      vec!["--steps".into(), "4".into(), "--backend".into(), "cpu".into()],
    ),
  };

  RuntimeProfile {
    hardware: hardware_profile.hardware,
    accelerator: hardware_profile.accelerator,
    expected_performance: hardware_profile.expected_performance,
    llm_flags,
    image_flags,
    model_directory,
    cache_directory,
    safety_prompt: SAFETY_PROMPT.to_string(),
    style_wrapper: STYLE_WRAPPER.to_string(),
    llm_binary: paths.llm_binary.display().to_string(),
    image_binary: paths.image_binary.display().to_string(),
    llm_model: paths.llm_model.display().to_string(),
    image_model: paths.image_model.display().to_string(),
    database_path: paths.database_path.display().to_string(),
  }
}

#[tauri::command]
fn get_model_status() -> Result<ModelStatus, String> {
  let paths = RuntimePaths::resolve();
  ensure_storage(&paths)?;

  Ok(ModelStatus {
    model_directory: paths.model_dir.display().to_string(),
    cache_directory: paths.cache_dir.display().to_string(),
    llm_binary: paths.llm_binary.display().to_string(),
    image_binary: paths.image_binary.display().to_string(),
    llm_model: paths.llm_model.display().to_string(),
    image_model: paths.image_model.display().to_string(),
    llm_ready: paths.llm_binary.exists() && paths.llm_model.exists(),
    image_ready: paths.image_binary.exists() && paths.image_model.exists(),
    database_path: paths.database_path.display().to_string(),
  })
}

#[tauri::command]
fn get_catalog(locale: Option<String>) -> Result<CatalogResponse, String> {
  let paths = RuntimePaths::resolve();
  let connection = open_database(&paths)?;
  let language = normalize_locale(locale.as_deref());
  load_catalog(&connection, language)
}

#[tauri::command]
fn generate_article(request: ArticleRequest) -> Result<GeneratedArticle, String> {
  let paths = RuntimePaths::resolve();
  let connection = open_database(&paths)?;
  let language = normalize_locale(request.locale.as_deref());
  let article = load_article(&connection, &request.article_id, language)?;
  let cache_key = format!("text:{}:{}", language, article.id);

  if let Some(text) = read_cache(&connection, &cache_key)? {
    return Ok(GeneratedArticle {
      article,
      generated_text: text,
      source: "cache".to_string(),
      prompt: String::new(),
      cached: true,
    });
  }

  let prompt = build_article_prompt(&article, language);
  let profile = get_runtime_profile();
  let generated_text = if paths.llm_binary.exists() && paths.llm_model.exists() {
    run_llama_sidecar(&paths, &profile.llm_flags, &prompt).unwrap_or_else(|error| fallback_article_text(&article, language, Some(&error)))
  } else {
    fallback_article_text(&article, language, None)
  };

  write_cache(&connection, &cache_key, &generated_text)?;

  Ok(GeneratedArticle {
    article,
    generated_text,
    source: if paths.llm_binary.exists() && paths.llm_model.exists() { "llama.cpp" } else { "fallback" }.to_string(),
    prompt,
    cached: false,
  })
}

#[tauri::command]
fn generate_image(request: ArticleRequest) -> Result<GeneratedImage, String> {
  let paths = RuntimePaths::resolve();
  let connection = open_database(&paths)?;
  let language = normalize_locale(request.locale.as_deref());
  let article = load_article(&connection, &request.article_id, language)?;
  let cache_key = format!("image:{}:{}", language, article.id);
  let prompt = STYLE_WRAPPER.replace("[SUBJECT]", &article.title);

  if let Some(path) = read_cache(&connection, &cache_key)? {
    if Path::new(&path).exists() {
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
  let output_path = paths.image_cache_dir.join(format!("{}-{}.svg", article.id, now_millis()));
  let profile = get_runtime_profile();

  let source = if paths.image_binary.exists() && paths.image_model.exists() {
    let generated_path = output_path.with_extension("png");
    match run_stable_diffusion_sidecar(&paths, &profile.image_flags, &prompt, &generated_path) {
      Ok(()) => {
        write_cache(&connection, &cache_key, &generated_path.display().to_string())?;
        return Ok(GeneratedImage {
          article_id: article.id,
          prompt,
          source: "stable-diffusion.cpp".to_string(),
          image_path: generated_path.display().to_string(),
          cached: false,
        });
      }
      Err(error) => format!("fallback ({error})"),
    }
  } else {
    "fallback".to_string()
  };

  write_placeholder_svg(&output_path, &article, language)?;
  write_cache(&connection, &cache_key, &output_path.display().to_string())?;

  Ok(GeneratedImage {
    article_id: article.id,
    prompt,
    source,
    image_path: output_path.display().to_string(),
    cached: false,
  })
}

#[derive(Clone)]
struct RuntimePaths {
  data_dir: PathBuf,
  model_dir: PathBuf,
  cache_dir: PathBuf,
  image_cache_dir: PathBuf,
  database_path: PathBuf,
  llm_binary: PathBuf,
  image_binary: PathBuf,
  llm_model: PathBuf,
  image_model: PathBuf,
}

impl RuntimePaths {
  fn resolve() -> Self {
    let data_dir = resolve_data_directory();
    let model_dir = data_dir.join("models");
    let cache_dir = data_dir.join("cache");
    let bin_dir = data_dir.join("bin");

    Self {
      data_dir: data_dir.clone(),
      model_dir: model_dir.clone(),
      cache_dir: cache_dir.clone(),
      image_cache_dir: cache_dir.join("images"),
      database_path: data_dir.join(DB_FILE_NAME),
      llm_binary: bin_dir.join(executable_name("llama-cli")),
      image_binary: bin_dir.join(executable_name("sd")),
      llm_model: model_dir.join(LLM_MODEL_FILE),
      image_model: model_dir.join(IMAGE_MODEL_FILE),
    }
  }
}

fn executable_name(name: &str) -> String {
  if cfg!(target_os = "windows") {
    format!("{name}.exe")
  } else {
    name.to_string()
  }
}

fn resolve_data_directory() -> PathBuf {
  if cfg!(target_os = "windows") {
    if let Ok(app_data) = env::var("APPDATA") {
      return PathBuf::from(app_data).join("Odyssee");
    }
  }

  env::var("HOME")
    .map(|home| PathBuf::from(home).join(".local/share/Odyssee"))
    .unwrap_or_else(|_| PathBuf::from("Odyssee"))
}

fn ensure_storage(paths: &RuntimePaths) -> Result<(), String> {
  fs::create_dir_all(&paths.data_dir).map_err(|error| error.to_string())?;
  fs::create_dir_all(&paths.model_dir).map_err(|error| error.to_string())?;
  fs::create_dir_all(&paths.cache_dir).map_err(|error| error.to_string())?;
  fs::create_dir_all(&paths.image_cache_dir).map_err(|error| error.to_string())?;
  Ok(())
}

fn open_database(paths: &RuntimePaths) -> Result<Connection, String> {
  ensure_storage(paths)?;
  let connection = Connection::open(&paths.database_path).map_err(|error| error.to_string())?;
  initialize_database(&connection)?;
  Ok(connection)
}

fn initialize_database(connection: &Connection) -> Result<(), String> {
  connection
    .execute_batch(
      "CREATE TABLE IF NOT EXISTS domains (
        id TEXT PRIMARY KEY,
        icon TEXT NOT NULL,
        color TEXT NOT NULL,
        name_fr TEXT NOT NULL,
        name_en TEXT NOT NULL,
        welcome_fr TEXT NOT NULL,
        welcome_en TEXT NOT NULL
      );
      CREATE TABLE IF NOT EXISTS sections (
        id TEXT NOT NULL,
        domain_id TEXT NOT NULL,
        article_id TEXT NOT NULL,
        icon TEXT NOT NULL,
        name_fr TEXT NOT NULL,
        name_en TEXT NOT NULL,
        position INTEGER NOT NULL,
        PRIMARY KEY (id, domain_id)
      );
      CREATE TABLE IF NOT EXISTS articles (
        id TEXT PRIMARY KEY,
        title_fr TEXT NOT NULL,
        title_en TEXT NOT NULL,
        summary_fr TEXT NOT NULL,
        summary_en TEXT NOT NULL,
        questions_fr TEXT NOT NULL,
        questions_en TEXT NOT NULL
      );
      CREATE TABLE IF NOT EXISTS cache_entries (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL,
        created_at INTEGER NOT NULL
      );",
    )
    .map_err(|error| error.to_string())?;

  seed_database(connection)
}

fn seed_database(connection: &Connection) -> Result<(), String> {
  for domain in DOMAINS {
    connection
      .execute(
        "INSERT OR REPLACE INTO domains (id, icon, color, name_fr, name_en, welcome_fr, welcome_en) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![domain.id, domain.icon, domain.color, domain.name_fr, domain.name_en, domain.welcome_fr, domain.welcome_en],
      )
      .map_err(|error| error.to_string())?;
  }

  for (position, section) in SECTIONS.iter().enumerate() {
    connection
      .execute(
        "INSERT OR REPLACE INTO sections (id, domain_id, article_id, icon, name_fr, name_en, position) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![section.id, section.domain_id, section.article_id, section.icon, section.name_fr, section.name_en, position as i64],
      )
      .map_err(|error| error.to_string())?;
  }

  for article in ARTICLES {
    connection
      .execute(
        "INSERT OR REPLACE INTO articles (id, title_fr, title_en, summary_fr, summary_en, questions_fr, questions_en) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
          article.id,
          article.title_fr,
          article.title_en,
          article.summary_fr,
          article.summary_en,
          article.questions_fr.join("\n"),
          article.questions_en.join("\n"),
        ],
      )
      .map_err(|error| error.to_string())?;
  }

  Ok(())
}

fn normalize_locale(locale: Option<&str>) -> &'static str {
  match locale.unwrap_or("fr").to_lowercase().as_str() {
    "en" | "en-us" | "en-gb" => "en",
    _ => "fr",
  }
}

fn localized(fr: String, en: String, locale: &str) -> String {
  if locale == "en" { en } else { fr }
}

fn load_catalog(connection: &Connection, locale: &str) -> Result<CatalogResponse, String> {
  let mut statement = connection
    .prepare("SELECT id, icon, color, name_fr, name_en, welcome_fr, welcome_en FROM domains ORDER BY rowid")
    .map_err(|error| error.to_string())?;
  let rows = statement
    .query_map([], |row| {
      let id: String = row.get(0)?;
      Ok((
        id,
        row.get::<_, String>(1)?,
        row.get::<_, String>(2)?,
        row.get::<_, String>(3)?,
        row.get::<_, String>(4)?,
        row.get::<_, String>(5)?,
        row.get::<_, String>(6)?,
      ))
    })
    .map_err(|error| error.to_string())?;

  let mut domains = Vec::new();
  for row in rows {
    let (id, icon, color, name_fr, name_en, welcome_fr, welcome_en) = row.map_err(|error| error.to_string())?;
    domains.push(DomainDto {
      sections: load_sections(connection, &id, locale)?,
      id,
      icon,
      color,
      name: localized(name_fr, name_en, locale),
      welcome: localized(welcome_fr, welcome_en, locale),
    });
  }

  Ok(CatalogResponse { domains })
}

fn load_sections(connection: &Connection, domain_id: &str, locale: &str) -> Result<Vec<SectionDto>, String> {
  let mut statement = connection
    .prepare("SELECT id, article_id, icon, name_fr, name_en FROM sections WHERE domain_id = ?1 ORDER BY position")
    .map_err(|error| error.to_string())?;
  let rows = statement
    .query_map(params![domain_id], |row| {
      Ok((
        row.get::<_, String>(0)?,
        row.get::<_, String>(1)?,
        row.get::<_, String>(2)?,
        row.get::<_, String>(3)?,
        row.get::<_, String>(4)?,
      ))
    })
    .map_err(|error| error.to_string())?;

  let mut sections = Vec::new();
  for row in rows {
    let (id, article_id, icon, name_fr, name_en) = row.map_err(|error| error.to_string())?;
    sections.push(SectionDto {
      id,
      article_id,
      icon,
      name: localized(name_fr, name_en, locale),
    });
  }

  Ok(sections)
}

fn load_article(connection: &Connection, article_id: &str, locale: &str) -> Result<ArticleDto, String> {
  connection
    .query_row(
      "SELECT id, title_fr, title_en, summary_fr, summary_en, questions_fr, questions_en FROM articles WHERE id = ?1",
      params![article_id],
      |row| {
        let questions_fr: String = row.get(5)?;
        let questions_en: String = row.get(6)?;
        Ok(ArticleDto {
          id: row.get(0)?,
          title: localized(row.get(1)?, row.get(2)?, locale),
          summary: localized(row.get(3)?, row.get(4)?, locale),
          questions: localized(questions_fr, questions_en, locale)
            .lines()
            .map(|line| line.to_string())
            .collect(),
        })
      },
    )
    .optional()
    .map_err(|error| error.to_string())?
    .ok_or_else(|| format!("Article inconnu: {article_id}"))
}

fn read_cache(connection: &Connection, key: &str) -> Result<Option<String>, String> {
  connection
    .query_row("SELECT value FROM cache_entries WHERE key = ?1", params![key], |row| row.get(0))
    .optional()
    .map_err(|error| error.to_string())
}

fn write_cache(connection: &Connection, key: &str, value: &str) -> Result<(), String> {
  connection
    .execute(
      "INSERT OR REPLACE INTO cache_entries (key, value, created_at) VALUES (?1, ?2, ?3)",
      params![key, value, now_millis() as i64],
    )
    .map_err(|error| error.to_string())?;
  Ok(())
}

fn build_article_prompt(article: &ArticleDto, locale: &str) -> String {
  let language_instruction = if locale == "en" {
    "Answer in English for a child aged 6 to 10. Use short paragraphs and end with exactly three curiosity questions."
  } else {
    "Réponds en français pour un enfant de 6 à 10 ans. Utilise des paragraphes courts et termine par exactement trois questions de curiosité."
  };

  format!(
    "{SAFETY_PROMPT}\n\n{language_instruction}\n\nSujet: {}\nRésumé fiable: {}\nQuestions proposées: {}",
    article.title,
    article.summary,
    article.questions.join(" | ")
  )
}

fn fallback_article_text(article: &ArticleDto, locale: &str, sidecar_error: Option<&str>) -> String {
  let intro = if locale == "en" {
    "Local demo explanation"
  } else {
    "Explication locale de démonstration"
  };
  let curiosity_label = if locale == "en" { "Curiosity paths" } else { "Pistes de curiosité" };
  let sidecar_note = sidecar_error
    .map(|error| format!("\n\nMode fallback actif: {error}"))
    .unwrap_or_default();

  format!(
    "{intro}: {}\n\n{}\n\n{curiosity_label}:\n• {}\n• {}\n• {}{}",
    article.title,
    article.summary,
    article.questions.get(0).cloned().unwrap_or_default(),
    article.questions.get(1).cloned().unwrap_or_default(),
    article.questions.get(2).cloned().unwrap_or_default(),
    sidecar_note
  )
}

fn run_llama_sidecar(paths: &RuntimePaths, flags: &[String], prompt: &str) -> Result<String, String> {
  let output = Command::new(&paths.llm_binary)
    .arg("-m")
    .arg(&paths.llm_model)
    .arg("-p")
    .arg(prompt)
    .arg("-n")
    .arg("420")
    .args(flags)
    .stdin(Stdio::null())
    .output()
    .map_err(|error| error.to_string())?;

  if !output.status.success() {
    return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
  }

  Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn run_stable_diffusion_sidecar(paths: &RuntimePaths, flags: &[String], prompt: &str, output_path: &Path) -> Result<(), String> {
  let output = Command::new(&paths.image_binary)
    .arg("-m")
    .arg(&paths.image_model)
    .arg("-p")
    .arg(prompt)
    .arg("-o")
    .arg(output_path)
    .args(flags)
    .stdin(Stdio::null())
    .output()
    .map_err(|error| error.to_string())?;

  if output.status.success() && output_path.exists() {
    Ok(())
  } else {
    Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
  }
}

fn write_placeholder_svg(output_path: &Path, article: &ArticleDto, locale: &str) -> Result<(), String> {
  let label = if locale == "en" { "offline illustration" } else { "illustration hors ligne" };
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
  <text x="480" y="430" text-anchor="middle" font-family="Arial, sans-serif" font-size="20" fill="#64748b">stable-diffusion.cpp prêt dès que le modèle est installé</text>
</svg>"##
  );

  fs::write(output_path, svg).map_err(|error| error.to_string())
}

fn escape_xml(value: &str) -> String {
  value
    .replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
    .replace('"', "&quot;")
    .replace('\'', "&apos;")
}

fn now_millis() -> u128 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|duration| duration.as_millis())
    .unwrap_or_default()
}

fn has_command(command: &str) -> bool {
  Command::new(command)
    .arg("--help")
    .output()
    .map(|result| result.status.success())
    .unwrap_or(false)
}

fn resolve_model_directory() -> String {
  RuntimePaths::resolve().model_dir.display().to_string()
}

fn resolve_cache_directory() -> String {
  RuntimePaths::resolve().cache_dir.display().to_string()
}

}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      // Register backend logging for both debug and packaged builds.
      app.handle().plugin(
        tauri_plugin_log::Builder::default()
          .level(log::LevelFilter::Info)
          .build(),
      )?;
      log::info!("Odyssée Kids backend logging initialized");
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      commands::detect_hardware,
      commands::get_runtime_profile,
      commands::get_model_status,
      commands::prepare_models,
      commands::clear_generation_cache,
      commands::get_catalog,
      commands::generate_article,
      commands::generate_image
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
  use crate::constants::{LLM_MODEL_FILE, SAFETY_PROMPT};
  use crate::generation::build_article_prompt;
  use crate::hardware::build_runtime_profile;
  use crate::models::ArticleDto;
  use crate::seeds::{ARTICLES, DOMAINS, SECTIONS};

  #[test]
  fn runtime_profile_includes_guardrails() {
    let profile = build_runtime_profile();
    assert!(profile.safety_prompt.contains("children encyclopedia"));
    assert!(profile.style_wrapper.contains("[SUBJECT]"));
    assert!(profile.llm_model.contains(LLM_MODEL_FILE));
  }

  #[test]
  fn runtime_profile_has_storage_paths() {
    let profile = build_runtime_profile();
    assert!(!profile.model_directory.is_empty());
    assert!(!profile.cache_directory.is_empty());
    assert!(!profile.database_path.is_empty());
  }

  #[test]
  fn catalog_seed_has_required_tree() {
    assert_eq!(DOMAINS.len(), 6);
    assert_eq!(SECTIONS.len(), 18);
    assert_eq!(ARTICLES.len(), 18);
  }

  #[test]
  fn prompt_contains_guardrails_and_subject() {
    let article = ArticleDto {
      id: "arctic-fox".to_string(),
      title: "Renard polaire".to_string(),
      summary: "Résumé".to_string(),
      questions: vec!["Question 1".to_string(), "Question 2".to_string(), "Question 3".to_string()],
    };

    let prompt = build_article_prompt(&article, "fr", None);
    assert!(prompt.contains(SAFETY_PROMPT));
    assert!(prompt.contains("Renard polaire"));
  }
}
