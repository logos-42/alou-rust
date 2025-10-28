import { ref, computed } from 'vue'
import { messages, walletMessages, type Language, type MessageKey } from '@/i18n/messages'

const currentLanguage = ref<Language>('zh')

export function useI18n() {
  const t = (key: MessageKey): string => {
    const zhMessages = { ...messages.zh, ...walletMessages.zh }
    const enMessages = { ...messages.en, ...walletMessages.en }
    const currentMessages = currentLanguage.value === 'zh' ? zhMessages : enMessages
    return currentMessages[key] || key
  }

  const setLanguage = (lang: Language) => {
    currentLanguage.value = lang
    localStorage.setItem('alou-language', lang)
  }

  const initLanguage = () => {
    const saved = localStorage.getItem('alou-language') as Language
    if (saved && (saved === 'zh' || saved === 'en')) {
      currentLanguage.value = saved
    }
  }

  return {
    t,
    currentLanguage: computed(() => currentLanguage.value),
    setLanguage,
    initLanguage
  }
}
