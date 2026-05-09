import { createI18n } from 'vue-i18n'

import en from './locales/en.json'
import fr from './locales/fr.json'

export const DEFAULT_LOCALE = 'fr'
export const LOCALE_STORAGE_KEY = 'odyssee.locale'

const SUPPORTED_LOCALES = ['en', 'fr']

function isSupportedLocale(locale) {
  return SUPPORTED_LOCALES.includes(locale)
}

function readSavedLocale() {
  const savedLocale = window.localStorage.getItem(LOCALE_STORAGE_KEY)

  return isSupportedLocale(savedLocale) ? savedLocale : DEFAULT_LOCALE
}

export function persistLocale(locale) {
  if (isSupportedLocale(locale)) {
    window.localStorage.setItem(LOCALE_STORAGE_KEY, locale)
  }
}

export const i18n = createI18n({
  legacy: false,
  locale: readSavedLocale(),
  messages: {
    en,
    fr,
  },
})
