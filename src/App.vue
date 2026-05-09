<script setup>
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { convertFileSrc, invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { persistLocale } from './i18n'
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

const catalog = ref([])
const selectedDomain = ref(null)
const selectedSection = ref(null)
const articleResult = ref(null)
const questionsResult = ref(null)
const subcategoriesResult = ref(null)
const imageResult = ref(null)
const selectedCuriosityQuestion = ref('')
const articleNavigationTrail = ref([])
const selectedAgeRange = ref('6-10')
const errorMessage = ref('')
const loadingCatalog = ref(false)
const loadingArticle = ref(false)
const loadingQuestions = ref(false)
const loadingSubcategories = ref(false)
const loadingImage = ref(false)
const preparingModels = ref(false)
const clearingCache = ref(false)
const modelPreparationMessage = ref('')
const modelPreparationError = ref('')
const systemActionMessage = ref('')
const systemActionError = ref('')
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
  llm_binary_ready: false,
  image_binary_ready: false,
  llm_model_ready: false,
  image_model_ready: false,
  llm_model: 'N/A',
  image_model: 'N/A',
  downloads: null,
})

const availableLocales = [
  { value: 'en', labelKey: 'app.locales.en' },
  { value: 'fr', labelKey: 'app.locales.fr' },
]

const availableAgeRanges = [
  { value: '3-6', labelKey: 'app.ageRanges.age3_6' },
  { value: '6-10', labelKey: 'app.ageRanges.age6_10' },
  { value: '10-14', labelKey: 'app.ageRanges.age10_14' },
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
  return subcategoriesResult.value?.sections ?? selectedDomain.value?.sections ?? []
})

const articleBreadcrumbs = computed(() => {
  const breadcrumbs = [{ label: t('app.breadcrumbs.domains'), target: 'domains' }]

  if (selectedDomain.value) {
    breadcrumbs.push({ label: selectedDomain.value.name, target: 'topics' })
  }

  if (selectedSection.value) {
    breadcrumbs.push({ label: selectedSection.value.name, target: 'article', questionIndex: -1 })
  }

  articleNavigationTrail.value.forEach((question, questionIndex) => {
    breadcrumbs.push({ label: question, target: 'question', questionIndex })
  })

  return breadcrumbs
})

const article = computed(() => articleResult.value?.article ?? null)
const curiosityQuestions = computed(() => questionsResult.value?.questions ?? article.value?.questions ?? [])
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

const sourceLabelKeys = {
  SQLite: 'app.sources.sqlite',
  catalog: 'app.sources.catalog',
  cache: 'app.sources.cache',
  'llama.cpp': 'app.sources.llama',
  'stable-diffusion.cpp': 'app.sources.stableDiffusion',
}

function sourceLabel(source) {
  const labelKey = sourceLabelKeys[source]

  return labelKey ? t(labelKey) : source
}

const selectedSourceLabel = computed(() => {
  if (!articleResult.value && !questionsResult.value && !subcategoriesResult.value && !imageResult.value) {
    return sourceLabel('SQLite')
  }

  return [articleResult.value?.source, questionsResult.value?.source, subcategoriesResult.value?.source, imageResult.value?.source]
    .filter(Boolean)
    .map(sourceLabel)
    .join(' / ')
})

const selectedAgeRangeLabel = computed(() => {
  const ageRange = availableAgeRanges.find((item) => item.value === selectedAgeRange.value)

  return ageRange ? t(ageRange.labelKey) : selectedAgeRange.value
})

const runtimeAssets = computed(() => {
  const downloads = modelStatus.value.downloads ?? {}
  return [
    {
      label: t('app.runtime.assets.llmBinary'),
      ready: modelStatus.value.llm_binary_ready,
      path: systemProfile.value.llm_binary,
      detail: downloads.llm_binary,
    },
    {
      label: t('app.runtime.assets.imageBinary'),
      ready: modelStatus.value.image_binary_ready,
      path: systemProfile.value.image_binary,
      detail: downloads.image_binary,
    },
    {
      label: t('app.runtime.assets.llmModel'),
      ready: modelStatus.value.llm_model_ready,
      path: modelStatus.value.llm_model,
      detail: downloads.llm,
    },
    {
      label: t('app.runtime.assets.imageModel'),
      ready: modelStatus.value.image_model_ready,
      path: modelStatus.value.image_model,
      detail: downloads.image,
    },
  ]
})

const readyRuntimeAssetCount = computed(() => runtimeAssets.value.filter((asset) => asset.ready).length)

const runtimePreparationSummary = computed(() => {
  const total = runtimeAssets.value.length
  const readyCount = readyRuntimeAssetCount.value

  if (preparingModels.value) {
    return t('app.runtime.summary.preparing', { message: modelPreparationMessage.value, ready: readyCount, total })
  }

  if (readyCount === total) {
    return t('app.runtime.summary.ready')
  }

  return t('app.runtime.summary.missing', { ready: readyCount, total })
})

const runtimeProgressPercent = computed(() => {
  if (!runtimeAssets.value.length) {
    return 0
  }

  return Math.round((readyRuntimeAssetCount.value / runtimeAssets.value.length) * 100)
})

const articleGenerationFeedback = computed(() => {
  if (loadingArticle.value) {
    return modelStatus.value.llm_ready
      ? t('app.generation.article.loading')
      : t('app.generation.article.missing')
  }

  if (articleResult.value?.source) {
    return t('app.generation.article.ready', { source: sourceLabel(articleResult.value.source) })
  }

  return t('app.generation.article.pending')
})

const questionsGenerationFeedback = computed(() => {
  if (loadingQuestions.value) {
    return modelStatus.value.llm_ready
      ? t('app.generation.questions.loading')
      : t('app.generation.questions.missing')
  }

  if (questionsResult.value?.source) {
    return t('app.generation.questions.ready', { source: sourceLabel(questionsResult.value.source) })
  }

  return t('app.generation.questions.pending')
})

const subcategoriesGenerationFeedback = computed(() => {
  if (loadingSubcategories.value) {
    return modelStatus.value.llm_ready
      ? t('app.generation.subcategories.loading')
      : t('app.generation.subcategories.missing')
  }

  if (subcategoriesResult.value?.source) {
    return t('app.generation.subcategories.ready', { source: subcategoriesResult.value.source })
  }

  return t('app.generation.subcategories.pending')
})

const imageGenerationFeedback = computed(() => {
  if (loadingImage.value) {
    return modelStatus.value.image_ready
      ? t('app.generation.image.loading')
      : t('app.generation.image.missing')
  }

  if (imageResult.value?.source) {
    return t('app.generation.image.ready', { source: sourceLabel(imageResult.value.source) })
  }

  return t('app.generation.image.pending')
})

function runtimeAssetState(asset) {
  if (asset.detail?.error) {
    return t('app.runtime.states.error')
  }

  if (asset.ready) {
    return asset.detail?.downloaded ? t('app.runtime.states.downloaded') : t('app.runtime.states.ready')
  }

  return preparingModels.value ? t('app.runtime.states.preparing') : t('app.runtime.states.missing')
}

function runtimeAssetFeedback(asset) {
  if (asset.detail?.error) {
    return t('app.runtime.feedback.error')
  }

  if (asset.ready) {
    return asset.detail?.downloaded ? t('app.runtime.feedback.downloaded') : t('app.runtime.feedback.ready')
  }

  if (preparingModels.value) {
    return t('app.runtime.feedback.preparing')
  }

  return t('app.runtime.feedback.missing')
}

function modelReadinessLabel(ready, modelReady, binaryReady) {
  if (ready) {
    return t('app.runtime.modelStates.ready')
  }

  return modelReady && !binaryReady
    ? t('app.runtime.modelStates.modelOnly')
    : t('app.runtime.modelStates.missing')
}

function runtimeAssetStateClass(asset) {
  if (asset.detail?.error) {
    return 'bg-amber-100 text-amber-800'
  }

  if (asset.ready) {
    return 'bg-emerald-100 text-emerald-800'
  }

  return 'bg-blue-100 text-blue-800'
}

function iconFor(iconName) {
  return iconRegistry[iconName] ?? Sparkles
}

function domainColor(domain) {
  return domainColors[domain.id] ?? domain.color ?? 'from-blue-400 to-indigo-500'
}

async function renderLoadingState() {
  await nextTick()
  await new Promise((resolve) => requestAnimationFrame(resolve))
}

async function selectDomain(domain) {
  selectedDomain.value = domain
  selectedSection.value = null
  articleResult.value = null
  questionsResult.value = null
  subcategoriesResult.value = null
  imageResult.value = null
  selectedCuriosityQuestion.value = ''
  articleNavigationTrail.value = []
  errorMessage.value = ''

  await generateSubcategories(domain.id)
}

async function selectSection(section) {
  selectedSection.value = section
  articleResult.value = null
  questionsResult.value = null
  imageResult.value = null
  selectedCuriosityQuestion.value = ''
  articleNavigationTrail.value = []
  errorMessage.value = ''

  await Promise.all([generateArticle(section.article_id), generateQuestions(section.article_id), generateImage(section.article_id)])
}

function backToTopics() {
  selectedSection.value = null
  articleResult.value = null
  questionsResult.value = null
  imageResult.value = null
  selectedCuriosityQuestion.value = ''
  articleNavigationTrail.value = []
}

function backToDomains() {
  selectedDomain.value = null
  selectedSection.value = null
  articleResult.value = null
  questionsResult.value = null
  subcategoriesResult.value = null
  imageResult.value = null
  selectedCuriosityQuestion.value = ''
  articleNavigationTrail.value = []
}

async function navigateArticleBreadcrumb(breadcrumb) {
  if (breadcrumb.target === 'domains') {
    backToDomains()
    return
  }

  if (breadcrumb.target === 'topics') {
    backToTopics()
    return
  }

  if (!selectedSection.value || loadingArticle.value || loadingQuestions.value || loadingImage.value) {
    return
  }

  const nextTrail = breadcrumb.target === 'question'
    ? articleNavigationTrail.value.slice(0, breadcrumb.questionIndex + 1)
    : []
  const nextQuestion = nextTrail.length ? nextTrail[nextTrail.length - 1] : ''

  articleNavigationTrail.value = nextTrail
  selectedCuriosityQuestion.value = nextQuestion
  errorMessage.value = ''
  await generateArticle(selectedSection.value.article_id, nextQuestion)
}

async function loadSystemProfile() {
  try {
    systemActionError.value = ''
    systemProfile.value = await invoke('get_runtime_profile', { locale: locale.value })
    modelStatus.value = await invoke('get_model_status')

    if (!modelStatus.value.llm_ready || !modelStatus.value.image_ready) {
      preparingModels.value = true
      modelPreparationMessage.value = t('app.runtime.installing')
      modelPreparationError.value = ''
      await renderLoadingState()
      const downloads = await invoke('prepare_models')
      modelPreparationMessage.value = t('app.runtime.verifying')
      const refreshedStatus = await invoke('get_model_status')
      modelStatus.value = { ...refreshedStatus, downloads }
    }
  } catch (error) {
    systemProfile.value = {
      hardware: t('app.runtime.browserPreview.hardware'),
      accelerator: t('app.runtime.browserPreview.accelerator'),
      expected_performance: t('app.runtime.browserPreview.performance'),
      model_directory: t('app.runtime.browserPreview.storagePath'),
      cache_directory: t('app.runtime.browserPreview.cachePath'),
      database_path: t('app.runtime.browserPreview.databasePath'),
      llm_binary: t('app.runtime.browserPreview.sidecarPath'),
      image_binary: t('app.runtime.browserPreview.sidecarPath'),
    }
    modelPreparationError.value = t('app.errors.runtimeUnavailable', { error })
  } finally {
    preparingModels.value = false
    modelPreparationMessage.value = ''
  }
}

async function clearGenerationCache() {
  clearingCache.value = true
  systemActionMessage.value = ''
  systemActionError.value = ''

  try {
    await renderLoadingState()
    const result = await invoke('clear_generation_cache')
    systemActionMessage.value = t('app.system.cacheCleared', {
      entries: result.entries_deleted,
      files: result.files_deleted,
    })
    articleResult.value = null
    questionsResult.value = null
    subcategoriesResult.value = null
    imageResult.value = null
    selectedCuriosityQuestion.value = ''
    articleNavigationTrail.value = []
  } catch (error) {
    systemActionError.value = t('app.system.cacheClearError', { error })
  } finally {
    clearingCache.value = false
  }
}

async function loadCatalog() {
  loadingCatalog.value = true
  errorMessage.value = ''

  try {
    await renderLoadingState()
    const response = await invoke('get_catalog', { locale: locale.value })
    catalog.value = response.domains

    if (selectedDomain.value) {
      selectedDomain.value = response.domains.find((domain) => domain.id === selectedDomain.value.id) ?? null
      subcategoriesResult.value = null
      if (selectedDomain.value) {
        await generateSubcategories(selectedDomain.value.id)
      }
    }
  } catch (error) {
    catalog.value = []
    selectedDomain.value = null
    selectedSection.value = null
    errorMessage.value = t('app.errors.previewMode', { error })
  } finally {
    loadingCatalog.value = false
  }
}

async function generateQuestions(articleId) {
  loadingQuestions.value = true

  try {
    await renderLoadingState()
    questionsResult.value = await invoke('generate_questions', {
      request: { article_id: articleId, locale: locale.value, age_range: selectedAgeRange.value },
    })
  } catch (error) {
    errorMessage.value = t('app.errors.questions', { error })
  } finally {
    loadingQuestions.value = false
  }
}

async function generateSubcategories(domainId) {
  loadingSubcategories.value = true

  try {
    await renderLoadingState()
    subcategoriesResult.value = await invoke('generate_subcategories', {
      request: { domain_id: domainId, locale: locale.value, age_range: selectedAgeRange.value },
    })
  } catch (error) {
    errorMessage.value = t('app.errors.subcategories', { error })
  } finally {
    loadingSubcategories.value = false
  }
}

async function generateArticle(articleId, curiosityQuestion = '') {
  loadingArticle.value = true

  try {
    const request = { article_id: articleId, locale: locale.value, age_range: selectedAgeRange.value }

    if (curiosityQuestion) {
      request.question = curiosityQuestion
    }

    await renderLoadingState()
    articleResult.value = await invoke('generate_article', {
      request,
    })
  } catch (error) {
    errorMessage.value = t('app.errors.article', { error })
  } finally {
    loadingArticle.value = false
  }
}

async function generateImage(articleId) {
  loadingImage.value = true

  try {
    await renderLoadingState()
    imageResult.value = await invoke('generate_image', {
      request: { article_id: articleId, locale: locale.value, age_range: selectedAgeRange.value },
    })
  } catch (error) {
    errorMessage.value = t('app.errors.image', { error })
  } finally {
    loadingImage.value = false
  }
}

async function regenerateCurrentArticle() {
  if (!selectedSection.value) {
    return
  }

  await Promise.all([
    generateArticle(selectedSection.value.article_id, selectedCuriosityQuestion.value),
    generateQuestions(selectedSection.value.article_id),
    generateImage(selectedSection.value.article_id),
  ])
}

async function answerCuriosityQuestion(question) {
  if (!selectedSection.value || loadingArticle.value) {
    return
  }

  articleNavigationTrail.value = [...articleNavigationTrail.value, question]
  selectedCuriosityQuestion.value = question
  errorMessage.value = ''
  await generateArticle(selectedSection.value.article_id, question)
}

onMounted(() => {
  loadSystemProfile()
  loadCatalog()
})

watch(locale, async () => {
  persistLocale(locale.value)
  await loadSystemProfile()
  await loadCatalog()

  if (selectedSection.value) {
    await regenerateCurrentArticle()
  }
})

watch(selectedAgeRange, async () => {
  articleNavigationTrail.value = []
  selectedCuriosityQuestion.value = ''
  subcategoriesResult.value = null

  if (selectedDomain.value) {
    await generateSubcategories(selectedDomain.value.id)
  }

  if (selectedSection.value) {
    await regenerateCurrentArticle()
  }
})
</script>

<template>
  <div class="mx-auto flex min-h-screen w-full max-w-7xl flex-col gap-6 px-4 py-8 sm:px-8">
    <div v-if="preparingModels" class="fixed inset-0 z-50 grid place-items-center bg-slate-950/70 px-4 backdrop-blur-sm">
      <section class="w-full max-w-3xl rounded-[2rem] bg-white p-6 shadow-2xl ring-1 ring-slate-200" role="status" aria-live="polite">
        <div class="flex items-start gap-4">
          <span class="rounded-2xl bg-blue-50 p-3 text-blue-700">
            <LoaderCircle class="h-7 w-7 animate-spin" />
          </span>
          <div>
            <p class="text-xs font-semibold uppercase tracking-wide text-blue-700">{{ t('app.runtime.firstLaunch') }}</p>
            <h2 class="mt-1 text-2xl font-black text-slate-950">{{ t('app.runtime.prepareTitle') }}</h2>
            <p class="mt-2 text-sm leading-relaxed text-slate-600">
              {{ t('app.runtime.prepareBody') }}
            </p>
          </div>
        </div>

        <div class="mt-5 rounded-2xl bg-blue-50 p-4 text-sm text-blue-900 ring-1 ring-blue-100">
          <div class="flex flex-wrap items-center justify-between gap-2">
            <p class="inline-flex items-center gap-2 font-semibold">
              <LoaderCircle class="h-4 w-4 animate-spin" /> {{ runtimePreparationSummary }}
            </p>
            <span class="rounded-full bg-white px-2 py-1 text-xs font-bold text-blue-700">
              {{ t('app.runtime.readyCount', { ready: readyRuntimeAssetCount, total: runtimeAssets.length }) }}
            </span>
          </div>
          <div class="mt-3 h-2 overflow-hidden rounded-full bg-white">
            <div class="h-full rounded-full bg-blue-500 transition-all" :style="{ width: `${runtimeProgressPercent}%` }" />
          </div>
          <p class="mt-2 text-xs text-blue-700">
            {{ t('app.runtime.firstLaunchNote') }}
          </p>
        </div>

        <div class="mt-5 grid gap-3 text-sm sm:grid-cols-2">
          <div class="rounded-2xl bg-slate-50 p-3">
            <p class="font-semibold text-slate-500">{{ t('app.runtime.detectedPlatform') }}</p>
            <p class="text-slate-900">{{ systemProfile.hardware }} · {{ systemProfile.accelerator }}</p>
          </div>
          <div class="rounded-2xl bg-slate-50 p-3">
            <p class="font-semibold text-slate-500">{{ t('app.runtime.localStorage') }}</p>
            <p class="break-all text-slate-900">{{ systemProfile.model_directory }}</p>
          </div>
        </div>

        <ol class="mt-5 space-y-3">
          <li
            v-for="asset in runtimeAssets"
            :key="asset.label"
            class="rounded-2xl border border-slate-200 bg-white p-3 text-sm"
          >
            <div class="flex flex-wrap items-center justify-between gap-2">
              <span class="font-semibold text-slate-900">{{ asset.label }}</span>
              <span :class="['rounded-full px-2 py-1 text-xs font-semibold', runtimeAssetStateClass(asset)]">
                {{ runtimeAssetState(asset) }}
              </span>
            </div>
            <p class="mt-2 break-all text-xs text-slate-500">{{ asset.path }}</p>
            <p class="mt-1 text-xs text-slate-600">{{ runtimeAssetFeedback(asset) }}</p>
            <p v-if="asset.detail?.url" class="mt-1 break-all text-xs text-slate-400">{{ asset.detail.url }}</p>
            <p v-if="asset.detail?.error" class="mt-2 text-xs font-medium text-amber-700">{{ asset.detail.error }}</p>
          </li>
        </ol>
      </section>
    </div>

    <header class="overflow-hidden rounded-[2rem] bg-white/90 p-6 shadow-xl ring-1 ring-slate-200 backdrop-blur">
      <div class="flex flex-wrap items-center justify-between gap-4">
        <div>
          <p class="mb-2 inline-flex items-center gap-2 rounded-full bg-blue-50 px-3 py-1 text-xs font-semibold uppercase tracking-wide text-blue-700">
            <Sparkles class="h-4 w-4" /> MVP local-first
          </p>
          <h1 class="text-4xl font-black tracking-tight text-slate-950">{{ t('app.title') }}</h1>
          <p class="mt-1 text-slate-600">{{ t('app.subtitle') }}</p>
        </div>

        <div class="flex flex-wrap items-center gap-3">
          <label class="flex items-center gap-2 rounded-xl bg-slate-100 px-3 py-2 text-sm">
            <span class="font-medium text-slate-700">{{ t('app.ageRange') }}</span>
            <select v-model="selectedAgeRange" class="rounded-lg border border-slate-300 bg-white px-2 py-1">
              <option
                v-for="ageRange in availableAgeRanges"
                :key="ageRange.value"
                :value="ageRange.value"
              >
                {{ t(ageRange.labelKey) }}
              </option>
            </select>
          </label>

          <label class="flex items-center gap-2 rounded-xl bg-slate-100 px-3 py-2 text-sm">
            <span class="font-medium text-slate-700">{{ t('app.language') }}</span>
            <select v-model="locale" class="rounded-lg border border-slate-300 bg-white px-2 py-1">
              <option
                v-for="localeItem in availableLocales"
                :key="localeItem.value"
                :value="localeItem.value"
              >
                {{ t(localeItem.labelKey) }}
              </option>
            </select>
          </label>
        </div>
      </div>

      <div class="mt-5 flex flex-wrap items-center gap-3 text-sm text-slate-500">
        <span class="rounded-full bg-slate-100 px-3 py-1 font-semibold text-slate-700">{{ t(`app.levels.${currentLevel}`) }}</span>
        <span class="rounded-full bg-blue-50 px-3 py-1 font-medium text-blue-700">{{ selectedAgeRangeLabel }}</span>
        <span class="rounded-full bg-emerald-50 px-3 py-1 font-medium text-emerald-700">{{ t('app.localOnly') }}</span>
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
              <h2 class="text-2xl font-bold text-slate-950">{{ t('app.domains.title') }}</h2>
              <p class="mt-1 text-sm text-slate-600">{{ t('app.domains.subtitle') }}</p>
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
              <p class="mt-1 text-sm text-slate-600">{{ t('app.topics.subtitle') }}</p>
            </div>
            <button
              type="button"
              class="rounded-lg bg-slate-100 px-3 py-2 text-sm font-medium text-slate-700 hover:bg-slate-200"
              @click="backToDomains"
            >
              {{ t('app.actions.goHome') }}
            </button>
          </div>

          <div class="rounded-2xl bg-blue-50 p-4 text-blue-900">
            <p>{{ selectedDomain.welcome }}</p>
            <p class="mt-2 inline-flex items-center gap-2 text-xs font-semibold text-blue-700">
              <LoaderCircle v-if="loadingSubcategories" class="h-4 w-4 animate-spin" /> {{ subcategoriesGenerationFeedback }}
            </p>
          </div>

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
          <nav class="flex flex-wrap items-center gap-2 text-sm" :aria-label="t('app.breadcrumbs.ariaLabel')">
            <template
              v-for="(breadcrumb, breadcrumbIndex) in articleBreadcrumbs"
              :key="`${breadcrumb.target}-${breadcrumbIndex}`"
            >
              <button
                type="button"
                class="max-w-[14rem] truncate rounded-full px-3 py-1 font-medium transition disabled:cursor-default"
                :class="breadcrumbIndex === articleBreadcrumbs.length - 1 ? 'bg-blue-600 text-white' : 'bg-slate-100 text-slate-600 hover:bg-blue-50 hover:text-blue-700'"
                :disabled="breadcrumbIndex === articleBreadcrumbs.length - 1 || loadingArticle || loadingQuestions || loadingImage"
                @click="navigateArticleBreadcrumb(breadcrumb)"
              >
                {{ breadcrumb.label }}
              </button>
              <span v-if="breadcrumbIndex < articleBreadcrumbs.length - 1" class="text-slate-300">/</span>
            </template>
          </nav>

          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm font-semibold uppercase tracking-wide text-blue-600">{{ selectedSection.name }}</p>
              <h2 class="text-2xl font-bold text-slate-950">{{ article?.title ?? t('app.generation.article.titleLoading') }}</h2>
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

          <div
            v-if="loadingArticle || loadingQuestions || loadingImage"
            class="rounded-2xl bg-blue-50 p-4 text-sm text-blue-900 ring-1 ring-blue-100"
            role="status"
            aria-live="polite"
          >
            <p class="inline-flex items-center gap-2 font-semibold">
              <LoaderCircle class="h-4 w-4 animate-spin" /> {{ t('app.generation.localInProgress') }}
            </p>
            <div class="mt-3 grid gap-2 sm:grid-cols-3">
              <p class="rounded-xl bg-white/70 px-3 py-2">{{ articleGenerationFeedback }}</p>
              <p class="rounded-xl bg-white/70 px-3 py-2">{{ questionsGenerationFeedback }}</p>
              <p class="rounded-xl bg-white/70 px-3 py-2">{{ imageGenerationFeedback }}</p>
            </div>
          </div>

          <div class="grid gap-5 lg:grid-cols-[1fr_1.1fr]">
            <div class="overflow-hidden rounded-3xl bg-gradient-to-br from-blue-100 to-purple-100 p-4 ring-1 ring-blue-100">
              <div class="flex items-center justify-between text-sm font-medium text-blue-700">
                <span>{{ t('app.actions.generateImage') }}</span>
                <span v-if="loadingImage" class="inline-flex items-center gap-1"><LoaderCircle class="h-4 w-4 animate-spin" /> {{ t('app.generation.image.ai') }}</span>
              </div>
              <img
                v-if="imageSource"
                :src="imageSource"
                :alt="t('app.generation.image.alt')"
                class="mt-3 aspect-[4/3] w-full rounded-2xl object-cover shadow-inner"
              >
              <div v-else class="mt-3 grid aspect-[4/3] place-items-center rounded-2xl bg-white/70 text-sm text-slate-500">
                {{ t('app.generation.image.placeholder') }}
              </div>
              <p class="mt-3 text-xs font-medium text-blue-700">{{ imageGenerationFeedback }}</p>
              <p class="mt-3 text-xs text-slate-600">{{ imageResult?.prompt }}</p>
            </div>

            <article class="rounded-3xl border border-slate-200 bg-slate-50 p-5">
              <div class="mb-3 flex items-center justify-between text-sm text-slate-500">
                <span>{{ t('app.generation.article.explanationLabel') }}</span>
                <span v-if="loadingArticle" class="inline-flex items-center gap-1"><LoaderCircle class="h-4 w-4 animate-spin" /> LLM</span>
              </div>
              <p class="whitespace-pre-line text-base leading-relaxed text-slate-800">
                {{ articleResult?.generated_text ?? article?.summary ?? t('app.generation.article.safePlaceholder') }}
              </p>
              <p class="mt-4 text-xs font-medium text-blue-700">{{ articleGenerationFeedback }}</p>
            </article>
          </div>

          <div>
            <h3 class="text-sm font-semibold uppercase tracking-wide text-slate-500">
              {{ t('app.actions.curiosityFeed') }}
            </h3>
            <div class="mt-3 flex flex-wrap gap-2">
              <button
                v-for="question in curiosityQuestions"
                :key="question"
                type="button"
                class="rounded-full px-4 py-2 text-sm transition disabled:cursor-not-allowed disabled:opacity-60"
                :class="selectedCuriosityQuestion === question ? 'bg-blue-600 text-white shadow' : 'bg-slate-100 text-slate-700 hover:bg-blue-100 hover:text-blue-900'"
                :disabled="loadingArticle || loadingQuestions"
                @click="answerCuriosityQuestion(question)"
              >
                {{ question }}
              </button>
            </div>
            <p class="mt-3 text-xs font-medium text-blue-700">{{ questionsGenerationFeedback }}</p>
          </div>

          <button
            type="button"
            class="rounded-xl bg-blue-600 px-4 py-2 text-sm font-semibold text-white shadow hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-60"
            :disabled="loadingArticle || loadingQuestions || loadingImage"
            @click="regenerateCurrentArticle"
          >
            {{ loadingArticle || loadingQuestions || loadingImage ? t('app.generation.regenerateLoading') : t('app.generation.regenerate') }}
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
            <dt class="font-medium text-slate-500">{{ t('app.system.cachePath') }}</dt>
            <dd class="break-all text-slate-900">{{ systemProfile.cache_directory }}</dd>
          </div>
          <div>
            <dt class="font-medium text-slate-500">{{ t('app.system.databasePath') }}</dt>
            <dd class="break-all text-slate-900">{{ systemProfile.database_path }}</dd>
          </div>
        </dl>

        <div class="mt-5 space-y-2">
          <button
            type="button"
            class="inline-flex w-full items-center justify-center gap-2 rounded-xl bg-slate-900 px-4 py-2 text-sm font-semibold text-white shadow hover:bg-slate-800 disabled:cursor-not-allowed disabled:opacity-60"
            :disabled="clearingCache"
            @click="clearGenerationCache"
          >
            <LoaderCircle v-if="clearingCache" class="h-4 w-4 animate-spin" />
            {{ t('app.system.clearCache') }}
          </button>
          <p v-if="systemActionMessage" class="text-xs font-medium text-emerald-700">{{ systemActionMessage }}</p>
          <p v-if="systemActionError" class="text-xs font-medium text-amber-700">{{ systemActionError }}</p>
        </div>

        <div class="mt-6 space-y-3 rounded-2xl bg-slate-50 p-4 text-xs text-slate-600">
          <p v-if="preparingModels" class="inline-flex items-center gap-1 font-medium text-blue-700">
            <LoaderCircle class="h-4 w-4 animate-spin" /> {{ t('app.runtime.onboarding') }}
          </p>
          <p v-if="modelPreparationError" class="font-medium text-amber-700">{{ modelPreparationError }}</p>
          <p><strong>{{ t('app.runtime.labels.llm') }}:</strong> {{ modelReadinessLabel(modelStatus.llm_ready, modelStatus.llm_model_ready, modelStatus.llm_binary_ready) }}</p>
          <p class="break-all"><strong>{{ t('app.runtime.labels.llmModel') }}:</strong> {{ modelStatus.llm_model }}</p>
          <p v-if="modelStatus.downloads?.llm_binary?.error" class="text-amber-700">{{ modelStatus.downloads.llm_binary.error }}</p>
          <p v-if="modelStatus.downloads?.llm?.error" class="text-amber-700">{{ modelStatus.downloads.llm.error }}</p>
          <p class="break-all"><strong>llama.cpp:</strong> {{ systemProfile.llm_binary }}</p>
          <p><strong>{{ t('app.runtime.labels.image') }}:</strong> {{ modelReadinessLabel(modelStatus.image_ready, modelStatus.image_model_ready, modelStatus.image_binary_ready) }}</p>
          <p class="break-all"><strong>{{ t('app.runtime.labels.imageModel') }}:</strong> {{ modelStatus.image_model }}</p>
          <p v-if="modelStatus.downloads?.image_binary?.error" class="text-amber-700">{{ modelStatus.downloads.image_binary.error }}</p>
          <p v-if="modelStatus.downloads?.image?.error" class="text-amber-700">{{ modelStatus.downloads.image.error }}</p>
          <p class="break-all"><strong>stable-diffusion.cpp:</strong> {{ systemProfile.image_binary }}</p>
          <p><strong>{{ t('app.system.promptGuard') }}:</strong> {{ systemProfile.safety_prompt }}</p>
          <p><strong>{{ t('app.system.styleWrapper') }}:</strong> {{ systemProfile.style_wrapper }}</p>
        </div>
      </aside>
    </main>
  </div>
</template>
