// ============================================================
// 灵境 (LingScape) 应用常量配置
// 来源：design.md §5 配置与常量
// ============================================================

// AI 模型标识
export const DOUBAO_VISION_MODEL = 'doubao-2.0-vision'
export const DOUBAO_LITE_MODEL = 'doubao-2.0-lite-32k'
export const SEEDREAM_MODEL = 'Seedream 4.0'
export const SEEDREAM_PREMIUM_MODEL = 'Seedream 5.0 lite'
export const JIMENG_VIDEO_MODEL = '即梦视频 3.0 Pro'

// 超时（毫秒）
export const AI_STEP1_TIMEOUT_MS = 3000
export const AI_STEP2_TIMEOUT_MS = 10000

// 默认值
export const DEFAULT_GENERATION_COUNT = 3
export const DEFAULT_RESOLUTION = '1080P'
export const MAX_GENERATION_COUNT = 5
export const VIDEO_FPS_DEFAULT = 30
export const VIDEO_CPU_MAX_PCT = 3
export const STATIC_CPU_MAX_PCT = 0.5
export const WALLPAPER_CACHE_LIMIT_GB = 5

// API Key
export const API_KEY_MASK_PREFIX = 'sk-****'

// i18n
export const DEFAULT_LOCALE = 'zh-CN'
export const FALLBACK_LOCALE = 'zh-CN'
export const I18N_BUNDLE_MAX_KB = 50

// 向导
export const ONBOARDING_WELCOME_DURATION_MS = 3000
export const ONBOARDING_QUICKSTART_DURATION_MS = 60000

// 火山引擎
export const VOLCANO_ENGINE_BASE_URL = 'https://ark.cn-beijing.volces.com/api/v3'
export const VOLCANO_REGISTER_URL = 'https://console.volcengine.com/ark/region:ark+cn-beijing/overview'

// 布局
export const SIDEBAR_WIDTH = 220
export const TITLEBAR_HEIGHT = 48
export const STATUSBAR_HEIGHT = 36
