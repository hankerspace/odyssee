<script setup>
import { computed, onMounted, ref, watch } from 'vue'
import { convertFileSrc, invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import {
  Bone,
  BookOpen,
  Bot,
  Brain,
  Brush,
  Compass,
  Cpu,
  Eye,
  Gem,
  HeartPulse,
  Landmark,
  LoaderCircle,
  Moon,
  Music,
  Orbit,
  Palette,
  PawPrint,
  Rocket,
  Sparkles,
  Sun,
  Train,
  TreePine,
  Trees,
  Waves,
  Zap,
} from 'lucide-vue-next'

const { locale, t } = useI18n()

const iconRegistry = {
  Bone,
  BookOpen,
  Bot,
  Brain,
  Brush,
  Compass,
  Cpu,
  Eye,
  Gem,
  HeartPulse,
  Landmark,
  Moon,
  Music,
  Orbit,
  Palette,
  PawPrint,
  Rocket,
  Sun,
  Train,
  TreePine,
  Trees,
  Waves,
  Zap,
}

const domainColors = {
  nature: 'from-emerald-400 to-teal-500',
  space: 'from-indigo-500 to-sky-500',
  history: 'from-amber-400 to-orange-500',
  technology: 'from-cyan-400 to-blue-500',
  'human-body': 'from-rose-400 to-pink-500',
  arts: 'from-purple-400 to-fuchsia-500',
}

const fallbackCatalog = [
  {
    id: 'nature',
    name: 'Nature',
    icon: 'Trees',
    color: domainColors.nature,
    welcome: 'Mode aperçu navigateur : lancez Tauri pour activer SQLite et les sidecars.',
    sections: [{ id: 'animals', name: 'Animaux', icon: 'PawPrint', article_id: 'arctic-fox' }],
  },
]

const catalog = ref([])
const selectedDomain = ref(null)
const selectedSection = ref(null)
const articleResult = ref(null)
const imageResult = ref(null)
const errorMessage = ref('')
const loadingCatalog = ref(false)
const loadingArticle = ref(false)
const loadingImage = ref(false)
const systemProfile = ref({
  hardware: 'Unknown',
  accelerator: 'Unknown',
  expected_performance: 'Unknown',
  model_directory: 'N/A',
  cache_directory: 'N/A',
  database_path: 'N/A',
  llm_binary: 'N/A',
  image_binary: 'N/A',
})
const modelStatus = ref({
  llm_ready: false,
  image_ready: false,
  llm_model: 'N/A',
  image_model: 'N/A',
})

const availableLocales = [
  { value: 'en', label: 'English' },
  { value: 'fr', label: 'Français' },
]

const currentLevel = computed(() => {
  if (selectedSection.value) {
    return 'article'
  }

  if (selectedDomain.value) {
    return 'topics'
  }

  return 'domains'
})

const selectedDomainSections = computed(() => {
  return selectedDomain.value?.sections ?? []
})

const article = computed(() => articleResult.value?.article ?? null)
const imageSource = computed(() => {
  const imagePath = imageResult.value?.image_path

  if (!imagePath) {
    return ''
  }

  if (imagePath.startsWith('data:') || imagePath.startsWith('http')) {
    return imagePath
  }

  return convertFileSrc(imagePath)
})

const selectedSourceLabel = computed(() => {
  if (!articleResult.value && !imageResult.value) {
    return 'SQLite + fallback local'
  }

  return [articleResult.value?.source, imageResult.value?.source].filter(Boolean).join(' / ')
})

function iconFor(iconName) {
  return iconRegistry[iconName] ?? Sparkles
}

function domainColor(domain) {
  return domainColors[domain.id] ?? domain.color ?? 'from-blue-400 to-indigo-500'
}

function selectDomain(domain) {
  selectedDomain.value = domain
  selectedSection.value = null
  articleResult.value = null
  imageResult.value = null
  errorMessage.value = ''
}

async function selectSection(section) {
  selectedSection.value = section
  articleResult.value = null
  imageResult.value = null
  errorMessage.value = ''

  await Promise.all([generateArticle(section.article_id), generateImage(section.article_id)])
}

function backToTopics() {
  selectedSection.value = null
  articleResult.value = null
  imageResult.value = null
}

function backToDomains() {
  selectedDomain.value = null
  selectedSection.value = null
  articleResult.value = null
  imageResult.value = null
}

async function loadSystemProfile() {
  try {
    systemProfile.value = await invoke('get_runtime_profile')
    modelStatus.value = await invoke('get_model_status')
  } catch {
    systemProfile.value = {
      hardware: 'Browser Preview',
      accelerator: 'Web',
      expected_performance: 'Use Tauri runtime for hardware detection',
      model_directory: 'Use Tauri runtime for storage path',
      cache_directory: 'Use Tauri runtime for cache path',
      database_path: 'Use Tauri runtime for SQLite path',
      llm_binary: 'Use Tauri runtime for sidecar path',
      image_binary: 'Use Tauri runtime for sidecar path',
    }
  }
}

async function loadCatalog() {
  loadingCatalog.value = true
  errorMessage.value = ''

  try {
    const response = await invoke('get_catalog', { locale: locale.value })
    catalog.value = response.domains

    if (selectedDomain.value) {
      selectedDomain.value = response.domains.find((domain) => domain.id === selectedDomain.value.id) ?? null
    }
  } catch (error) {
    catalog.value = fallbackCatalog
    errorMessage.value = `Mode aperçu: ${error}`
  } finally {
    loadingCatalog.value = false
  }
}

async function generateArticle(articleId) {
  loadingArticle.value = true

  try {
    articleResult.value = await invoke('generate_article', {
      request: { article_id: articleId, locale: locale.value },
    })
  } catch (error) {
    errorMessage.value = `Impossible de générer le texte: ${error}`
  } finally {
    loadingArticle.value = false
  }
}

async function generateImage(articleId) {
  loadingImage.value = true

  try {
    imageResult.value = await invoke('generate_image', {
      request: { article_id: articleId, locale: locale.value },
    })
  } catch (error) {
    errorMessage.value = `Impossible de générer l'image: ${error}`
  } finally {
    loadingImage.value = false
  }
}

async function regenerateCurrentArticle() {
  if (!selectedSection.value) {
    return
  }

  await Promise.all([
    generateArticle(selectedSection.value.article_id),
    generateImage(selectedSection.value.article_id),
  ])
}

onMounted(() => {
  loadSystemProfile()
  loadCatalog()
})

watch(locale, async () => {
  await loadCatalog()

  if (selectedSection.value) {
    await regenerateCurrentArticle()
  }
})
</script>

<template>
  <div class="mx-auto flex min-h-screen w-full max-w-7xl flex-col gap-6 px-4 py-8 sm:px-8">
    <header class="overflow-hidden rounded-[2rem] bg-white/90 p-6 shadow-xl ring-1 ring-slate-200 backdrop-blur">
      <div class="flex flex-wrap items-center justify-between gap-4">
        <div>
          <p class="mb-2 inline-flex items-center gap-2 rounded-full bg-blue-50 px-3 py-1 text-xs font-semibold uppercase tracking-wide text-blue-700">
            <Sparkles class="h-4 w-4" /> MVP local-first
          </p>
          <h1 class="text-4xl font-black tracking-tight text-slate-950">{{ t('app.title') }}</h1>
          <p class="mt-1 text-slate-600">{{ t('app.subtitle') }}</p>
        </div>

        <label class="flex items-center gap-2 rounded-xl bg-slate-100 px-3 py-2 text-sm">
          <span class="font-medium text-slate-700">{{ t('app.language') }}</span>
          <select v-model="locale" class="rounded-lg border border-slate-300 bg-white px-2 py-1">
            <option
              v-for="localeItem in availableLocales"
              :key="localeItem.value"
              :value="localeItem.value"
            >
              {{ localeItem.label }}
            </option>
          </select>
        </label>
      </div>

      <div class="mt-5 flex flex-wrap items-center gap-3 text-sm text-slate-500">
        <span class="rounded-full bg-slate-100 px-3 py-1 font-semibold text-slate-700">{{ t(`app.levels.${currentLevel}`) }}</span>
        <span class="rounded-full bg-emerald-50 px-3 py-1 font-medium text-emerald-700">100% local</span>
        <span class="rounded-full bg-purple-50 px-3 py-1 font-medium text-purple-700">{{ selectedSourceLabel }}</span>
      </div>
    </header>

    <main class="grid flex-1 gap-6 lg:grid-cols-[2fr_1fr]">
      <section class="rounded-3xl bg-white/95 p-6 shadow-lg ring-1 ring-slate-200">
        <div v-if="errorMessage" class="mb-5 rounded-2xl bg-amber-50 p-4 text-sm text-amber-900 ring-1 ring-amber-200">
          {{ errorMessage }}
        </div>

        <div v-if="currentLevel === 'domains'" class="space-y-6">
          <div class="flex items-end justify-between gap-4">
            <div>
              <h2 class="text-2xl font-bold text-slate-950">La roue des domaines</h2>
              <p class="mt-1 text-sm text-slate-600">Choisis une grande porte du savoir, puis zoome vers un thème.</p>
            </div>
            <LoaderCircle v-if="loadingCatalog" class="h-5 w-5 animate-spin text-blue-500" />
          </div>

          <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
            <button
              v-for="domain in catalog"
              :key="domain.id"
              type="button"
              class="group overflow-hidden rounded-3xl border border-slate-200 bg-white text-left shadow-sm transition hover:-translate-y-1 hover:border-blue-300 hover:shadow-xl"
              @click="selectDomain(domain)"
            >
              <span :class="['block bg-gradient-to-br p-5 text-white', domainColor(domain)]">
                <component :is="iconFor(domain.icon)" class="h-12 w-12 drop-shadow" />
              </span>
              <span class="block p-5">
                <span class="block text-xl font-bold text-slate-900">{{ domain.name }}</span>
                <span class="mt-2 line-clamp-3 text-sm text-slate-600">{{ domain.welcome }}</span>
              </span>
            </button>
          </div>
        </div>

        <div v-else-if="currentLevel === 'topics'" class="space-y-6">
          <div class="flex items-center justify-between">
            <div>
              <h2 class="text-2xl font-bold text-slate-950">{{ selectedDomain.name }}</h2>
              <p class="mt-1 text-sm text-slate-600">Exploration thématique guidée par l'arborescence SQLite.</p>
            </div>
            <button
              type="button"
              class="rounded-lg bg-slate-100 px-3 py-2 text-sm font-medium text-slate-700 hover:bg-slate-200"
              @click="backToDomains"
            >
              {{ t('app.actions.goHome') }}
            </button>
          </div>

          <p class="rounded-2xl bg-blue-50 p-4 text-blue-900">{{ selectedDomain.welcome }}</p>

          <div class="grid gap-3 sm:grid-cols-2">
            <button
              v-for="section in selectedDomainSections"
              :key="section.id"
              type="button"
              class="flex items-center gap-3 rounded-2xl border border-slate-200 bg-white px-4 py-4 text-left font-medium text-slate-800 transition hover:border-blue-300 hover:bg-blue-50"
              @click="selectSection(section)"
            >
              <component :is="iconFor(section.icon)" class="h-6 w-6 text-blue-600" />
              <span>{{ section.name }}</span>
            </button>
          </div>
        </div>

        <div v-else class="space-y-6">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm font-semibold uppercase tracking-wide text-blue-600">{{ selectedSection.name }}</p>
              <h2 class="text-2xl font-bold text-slate-950">{{ article?.title ?? 'Génération en cours…' }}</h2>
            </div>
            <div class="flex gap-2">
              <button
                type="button"
                class="rounded-lg bg-slate-100 px-3 py-2 text-sm font-medium text-slate-700 hover:bg-slate-200"
                @click="backToTopics"
              >
                {{ t('app.actions.back') }}
              </button>
              <button
                type="button"
                class="rounded-lg bg-slate-100 px-3 py-2 text-sm font-medium text-slate-700 hover:bg-slate-200"
                @click="backToDomains"
              >
                {{ t('app.actions.goHome') }}
              </button>
            </div>
          </div>

          <div class="grid gap-5 lg:grid-cols-[1fr_1.1fr]">
            <div class="overflow-hidden rounded-3xl bg-gradient-to-br from-blue-100 to-purple-100 p-4 ring-1 ring-blue-100">
              <div class="flex items-center justify-between text-sm font-medium text-blue-700">
                <span>{{ t('app.actions.generateImage') }}</span>
                <span v-if="loadingImage" class="inline-flex items-center gap-1"><LoaderCircle class="h-4 w-4 animate-spin" /> IA image</span>
              </div>
              <img
                v-if="imageSource"
                :src="imageSource"
                alt="Illustration générée localement"
                class="mt-3 aspect-[4/3] w-full rounded-2xl object-cover shadow-inner"
              >
              <div v-else class="mt-3 grid aspect-[4/3] place-items-center rounded-2xl bg-white/70 text-sm text-slate-500">
                Préparation de l'illustration…
              </div>
              <p class="mt-3 text-xs text-slate-600">{{ imageResult?.prompt }}</p>
            </div>

            <article class="rounded-3xl border border-slate-200 bg-slate-50 p-5">
              <div class="mb-3 flex items-center justify-between text-sm text-slate-500">
                <span>Explication adaptative 6-10 ans</span>
                <span v-if="loadingArticle" class="inline-flex items-center gap-1"><LoaderCircle class="h-4 w-4 animate-spin" /> LLM</span>
              </div>
              <p class="whitespace-pre-line text-base leading-relaxed text-slate-800">
                {{ articleResult?.generated_text ?? article?.summary ?? 'Le moteur prépare une fiche sûre et bienveillante.' }}
              </p>
            </article>
          </div>

          <div>
            <h3 class="text-sm font-semibold uppercase tracking-wide text-slate-500">
              {{ t('app.actions.curiosityFeed') }}
            </h3>
            <div class="mt-3 flex flex-wrap gap-2">
              <button
                v-for="question in article?.questions ?? []"
                :key="question"
                type="button"
                class="rounded-full bg-slate-100 px-4 py-2 text-sm text-slate-700 transition hover:bg-blue-100 hover:text-blue-900"
              >
                {{ question }}
              </button>
            </div>
          </div>

          <button
            type="button"
            class="rounded-xl bg-blue-600 px-4 py-2 text-sm font-semibold text-white shadow hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-60"
            :disabled="loadingArticle || loadingImage"
            @click="regenerateCurrentArticle"
          >
            Régénérer depuis les sidecars / cache
          </button>
        </div>
      </section>

      <aside class="rounded-3xl bg-white/95 p-6 shadow-lg ring-1 ring-slate-200">
        <h3 class="text-sm font-semibold uppercase tracking-wide text-slate-500">{{ t('app.system.profile') }}</h3>
        <dl class="mt-4 space-y-3 text-sm">
          <div>
            <dt class="font-medium text-slate-500">{{ t('app.system.hardware') }}</dt>
            <dd class="text-slate-900">{{ systemProfile.hardware }}</dd>
          </div>
          <div>
            <dt class="font-medium text-slate-500">{{ t('app.system.accelerator') }}</dt>
            <dd class="text-slate-900">{{ systemProfile.accelerator }}</dd>
          </div>
          <div>
            <dt class="font-medium text-slate-500">{{ t('app.system.performance') }}</dt>
            <dd class="text-slate-900">{{ systemProfile.expected_performance }}</dd>
          </div>
          <div>
            <dt class="font-medium text-slate-500">{{ t('app.system.modelPath') }}</dt>
            <dd class="break-all text-slate-900">{{ systemProfile.model_directory }}</dd>
          </div>
          <div>
            <dt class="font-medium text-slate-500">Cache local</dt>
            <dd class="break-all text-slate-900">{{ systemProfile.cache_directory }}</dd>
          </div>
          <div>
            <dt class="font-medium text-slate-500">Base SQLite</dt>
            <dd class="break-all text-slate-900">{{ systemProfile.database_path }}</dd>
          </div>
        </dl>

        <div class="mt-6 space-y-3 rounded-2xl bg-slate-50 p-4 text-xs text-slate-600">
          <p><strong>LLM:</strong> {{ modelStatus.llm_ready ? 'prêt' : 'fallback actif' }}</p>
          <p class="break-all"><strong>llama.cpp:</strong> {{ systemProfile.llm_binary }}</p>
          <p><strong>Image:</strong> {{ modelStatus.image_ready ? 'prêt' : 'placeholder SVG actif' }}</p>
          <p class="break-all"><strong>stable-diffusion.cpp:</strong> {{ systemProfile.image_binary }}</p>
          <p><strong>{{ t('app.system.promptGuard') }}:</strong> {{ systemProfile.safety_prompt }}</p>
          <p><strong>{{ t('app.system.styleWrapper') }}:</strong> {{ systemProfile.style_wrapper }}</p>
        </div>
      </aside>
    </main>
  </div>
</template>
