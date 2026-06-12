// ============================================================
// 灵境 i18n 国际化框架 (SET-005)
// vue-i18n 初始化、热切换、缺失 key 回退
// ============================================================
import { createI18n } from 'vue-i18n'
import zhCN from './locales/zh-CN.json'
import en from './locales/en.json'
import { DEFAULT_LOCALE, FALLBACK_LOCALE } from '@/config/constants'

// 格式化函数：按当前语言区域格式化数字/日期/货币
export function formatNumber(value: number, locale?: string): string {
  return new Intl.NumberFormat(locale ?? DEFAULT_LOCALE).format(value)
}

export function formatDate(value: Date | string | number, locale?: string): string {
  const date = typeof value === 'string' || typeof value === 'number' ? new Date(value) : value
  return new Intl.DateTimeFormat(locale ?? DEFAULT_LOCALE, {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  }).format(date)
}

export function formatCurrency(value: number, currency = 'CNY', locale?: string): string {
  return new Intl.NumberFormat(locale ?? DEFAULT_LOCALE, {
    style: 'currency',
    currency,
  }).format(value)
}

const i18n = createI18n({
  legacy: false, // Composition API 模式
  locale: DEFAULT_LOCALE,
  fallbackLocale: FALLBACK_LOCALE,
  missingWarn: true, // 开发环境 console.warn 提示缺失 key
  fallbackWarn: true,
  messages: {
    'zh-CN': zhCN,
    en,
  },
})

export default i18n
