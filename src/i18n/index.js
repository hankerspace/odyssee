import { createI18n } from 'vue-i18n'

import en from './locales/en.json'
import fr from './locales/fr.json'

const FALLBACK_LOCALE = 'en'

export const i18n = createI18n({
  legacy: false,
  locale: FALLBACK_LOCALE,
  fallbackLocale: FALLBACK_LOCALE,
  messages: {
    en,
    fr,
  },
})
