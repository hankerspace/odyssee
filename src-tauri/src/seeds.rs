//! Deterministic catalog seed data used to bootstrap the local SQLite tree.

#[derive(Clone, Copy)]
pub(crate) struct DomainSeed {
  pub(crate) id: &'static str,
  pub(crate) icon: &'static str,
  pub(crate) color: &'static str,
  pub(crate) name_fr: &'static str,
  pub(crate) name_en: &'static str,
  pub(crate) welcome_fr: &'static str,
  pub(crate) welcome_en: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) struct SectionSeed {
  pub(crate) id: &'static str,
  pub(crate) domain_id: &'static str,
  pub(crate) article_id: &'static str,
  pub(crate) icon: &'static str,
  pub(crate) name_fr: &'static str,
  pub(crate) name_en: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) struct ArticleSeed {
  pub(crate) id: &'static str,
  pub(crate) title_fr: &'static str,
  pub(crate) title_en: &'static str,
  pub(crate) summary_fr: &'static str,
  pub(crate) summary_en: &'static str,
  pub(crate) questions_fr: [&'static str; 3],
  pub(crate) questions_en: [&'static str; 3],
}

pub(crate) const DOMAINS: [DomainSeed; 6] = [
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

pub(crate) const SECTIONS: [SectionSeed; 18] = [
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

pub(crate) const ARTICLES: [ArticleSeed; 18] = [
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