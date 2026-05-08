<script setup>
import { computed, onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'

import { DOMAIN_CATALOG, SAFETY_GUARD, STYLE_WRAPPER } from './data/catalog'

const { locale, t, tm, te, messages } = useI18n()

const selectedDomain = ref(null)
const selectedArticle = ref(null)
const welcomePhrase = ref('')
const systemProfile = ref({
  hardware: 'Unknown',
  accelerator: 'Unknown',
  expected_performance: 'Unknown',
  model_directory: 'N/A',
})

const availableLocales = [
  { value: 'en', label: 'English' },
  { value: 'fr', label: 'Français' },
]

const currentLevel = computed(() => {
  if (selectedArticle.value) {
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

const articleTitle = computed(() => {
  if (!selectedArticle.value) {
    return ''
  }

  return getArticleText(selectedArticle.value, 'title')
})

const articleSummary = computed(() => {
  if (!selectedArticle.value) {
    return ''
  }

  return getArticleText(selectedArticle.value, 'summary')
})

const curiosityQuestions = computed(() => {
  if (!selectedArticle.value) {
    return []
  }

  return getArticleList(selectedArticle.value, 'questions')
})

const styledPrompt = computed(() => {
  if (!selectedArticle.value) {
    return STYLE_WRAPPER
  }

  return STYLE_WRAPPER.replace('[SUBJECT]', articleTitle.value)
})

function domainName(domainId) {
  return t(`domains.${domainId}.name`)
}

function sectionName(domainId, sectionId) {
  return t(`domains.${domainId}.sections.${sectionId}`)
}

function selectDomain(domain) {
  selectedDomain.value = domain
  selectedArticle.value = null

  const welcomePhrases = tm(`domains.${domain.id}.welcomePhrases`)
  const randomIndex = Math.floor(Math.random() * welcomePhrases.length)
  welcomePhrase.value = welcomePhrases[randomIndex]
}

function selectSection(section) {
  selectedArticle.value = section.article
}

function backToTopics() {
  selectedArticle.value = null
}

function backToDomains() {
  selectedDomain.value = null
  selectedArticle.value = null
  welcomePhrase.value = ''
}

function getArticleText(articleId, field) {
  const path = `articles.${articleId}.${field}`

  if (te(path)) {
    return t(path)
  }

  const fallbackMessage = messages.value.en?.articles?.[articleId]?.[field]
  return typeof fallbackMessage === 'string' ? fallbackMessage : ''
}

function getArticleList(articleId, field) {
  const path = `articles.${articleId}.${field}`

  if (te(path)) {
    return tm(path)
  }

  const fallbackMessage = messages.value.en?.articles?.[articleId]?.[field]
  return Array.isArray(fallbackMessage) ? fallbackMessage : []
}

async function loadSystemProfile() {
  try {
    const profile = await invoke('get_runtime_profile')
    systemProfile.value = {
      hardware: profile.hardware,
      accelerator: profile.accelerator,
      expected_performance: profile.expected_performance,
      model_directory: profile.model_directory,
    }
  } catch {
    systemProfile.value = {
      hardware: 'Browser Preview',
      accelerator: 'Web',
      expected_performance: 'Use Tauri runtime for hardware detection',
      model_directory: 'Use Tauri runtime for storage path',
    }
  }
}

onMounted(() => {
  loadSystemProfile()
})
</script>

<template>
  <div class="mx-auto flex min-h-screen w-full max-w-7xl flex-col gap-6 px-4 py-8 sm:px-8">
    <header class="rounded-3xl bg-white/90 p-6 shadow-lg ring-1 ring-slate-200">
      <div class="flex flex-wrap items-center justify-between gap-4">
        <div>
          <h1 class="text-3xl font-bold text-slate-900">{{ t('app.title') }}</h1>
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

      <div class="mt-4 text-sm text-slate-500">
        <span class="font-semibold text-slate-700">{{ t(`app.levels.${currentLevel}`) }}</span>
      </div>
    </header>

    <main class="grid flex-1 gap-6 lg:grid-cols-[2fr_1fr]">
      <section class="rounded-3xl bg-white/95 p-6 shadow-lg ring-1 ring-slate-200">
        <div v-if="currentLevel === 'domains'" class="space-y-6">
          <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
            <button
              v-for="domain in DOMAIN_CATALOG"
              :key="domain.id"
              type="button"
              class="group rounded-2xl border border-slate-200 bg-slate-50 p-5 text-left transition hover:-translate-y-1 hover:border-blue-300 hover:bg-blue-50"
              @click="selectDomain(domain)"
            >
              <component :is="domain.icon" class="h-9 w-9 text-blue-600" />
              <p class="mt-4 text-lg font-semibold text-slate-900">{{ domainName(domain.id) }}</p>
            </button>
          </div>
        </div>

        <div v-else-if="currentLevel === 'topics'" class="space-y-6">
          <div class="flex items-center justify-between">
            <h2 class="text-xl font-semibold text-slate-900">{{ domainName(selectedDomain.id) }}</h2>
            <button
              type="button"
              class="rounded-lg bg-slate-100 px-3 py-2 text-sm font-medium text-slate-700 hover:bg-slate-200"
              @click="backToDomains"
            >
              {{ t('app.actions.goHome') }}
            </button>
          </div>

          <p class="rounded-2xl bg-blue-50 p-4 text-blue-900">{{ welcomePhrase }}</p>

          <div class="grid gap-3 sm:grid-cols-2">
            <button
              v-for="section in selectedDomainSections"
              :key="section.id"
              type="button"
              class="rounded-xl border border-slate-200 bg-white px-4 py-3 text-left font-medium text-slate-800 transition hover:border-blue-300 hover:bg-blue-50"
              @click="selectSection(section)"
            >
              {{ sectionName(selectedDomain.id, section.id) }}
            </button>
          </div>
        </div>

        <div v-else class="space-y-6">
          <div class="flex items-center justify-between">
            <h2 class="text-xl font-semibold text-slate-900">{{ articleTitle }}</h2>
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

          <div class="rounded-2xl bg-gradient-to-br from-blue-100 to-purple-100 p-6">
            <p class="text-sm font-medium text-blue-700">{{ t('app.actions.generateImage') }}</p>
            <p class="mt-2 text-sm text-slate-600">{{ styledPrompt }}</p>
          </div>

          <p class="text-base leading-relaxed text-slate-700">{{ articleSummary }}</p>

          <div>
            <h3 class="text-sm font-semibold uppercase tracking-wide text-slate-500">
              {{ t('app.actions.curiosityFeed') }}
            </h3>
            <div class="mt-3 flex flex-wrap gap-2">
              <button
                v-for="question in curiosityQuestions"
                :key="question"
                type="button"
                class="rounded-full bg-slate-100 px-4 py-2 text-sm text-slate-700 transition hover:bg-blue-100 hover:text-blue-900"
              >
                {{ question }}
              </button>
            </div>
          </div>
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
        </dl>

        <div class="mt-6 space-y-3 rounded-2xl bg-slate-50 p-4 text-xs text-slate-600">
          <p><strong>{{ t('app.system.promptGuard') }}:</strong> {{ SAFETY_GUARD }}</p>
          <p><strong>{{ t('app.system.styleWrapper') }}:</strong> {{ STYLE_WRAPPER }}</p>
        </div>
      </aside>
    </main>
  </div>
</template>
