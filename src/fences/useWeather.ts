import { onMounted, onUnmounted, ref } from "vue";
import { apiFetch } from "../composables/apiFetch";

export interface WeatherSnapshot {
  city: string;
  tempC: number;
  feelsC: number;
  humidity: number;
  windKmh: number;
  code: number;
  label: string;
  icon: string;
  updatedAt: number;
}

const FALLBACK_CITY = "本地";
const FALLBACK_LAT = 39.9042;
const FALLBACK_LON = 116.4074;
const REFRESH_MS = 30 * 60 * 1000;

function weatherFromCode(code: number): { label: string; icon: string } {
  if (code === 0) return { label: "晴", icon: "☀" };
  if (code <= 3) return { label: "多云", icon: "⛅" };
  if (code === 45 || code === 48) return { label: "雾", icon: "〰" };
  if (code >= 51 && code <= 67) return { label: "雨", icon: "🌧" };
  if (code >= 71 && code <= 77) return { label: "雪", icon: "❄" };
  if (code >= 80 && code <= 82) return { label: "阵雨", icon: "🌦" };
  if (code >= 85 && code <= 86) return { label: "阵雪", icon: "❄" };
  if (code >= 95) return { label: "雷雨", icon: "⛈" };
  return { label: "天气", icon: "☁" };
}

async function resolveLocation(): Promise<{
  city: string;
  lat: number;
  lon: number;
}> {
  try {
    const res = await apiFetch("https://ipwho.is/");
    if (!res.ok) throw new Error(`geo ${res.status}`);
    const data = (await res.json()) as {
      success?: boolean;
      city?: string;
      latitude?: number;
      longitude?: number;
    };
    if (
      data.success !== false &&
      typeof data.latitude === "number" &&
      typeof data.longitude === "number"
    ) {
      return {
        city: (data.city || FALLBACK_CITY).trim() || FALLBACK_CITY,
        lat: data.latitude,
        lon: data.longitude,
      };
    }
  } catch {
    /* fallback below */
  }
  return { city: FALLBACK_CITY, lat: FALLBACK_LAT, lon: FALLBACK_LON };
}

async function fetchWeather(): Promise<WeatherSnapshot> {
  const loc = await resolveLocation();
  const url =
    `https://api.open-meteo.com/v1/forecast` +
    `?latitude=${loc.lat}&longitude=${loc.lon}` +
    `&current=temperature_2m,apparent_temperature,relative_humidity_2m,weather_code,wind_speed_10m` +
    `&timezone=auto&wind_speed_unit=kmh`;
  const res = await apiFetch(url);
  if (!res.ok) throw new Error(`weather ${res.status}`);
  const body = (await res.json()) as {
    current?: {
      temperature_2m?: number;
      apparent_temperature?: number;
      relative_humidity_2m?: number;
      weather_code?: number;
      wind_speed_10m?: number;
    };
  };
  const cur = body.current;
  if (!cur || typeof cur.temperature_2m !== "number") {
    throw new Error("weather payload incomplete");
  }
  const code = typeof cur.weather_code === "number" ? cur.weather_code : 0;
  const meta = weatherFromCode(code);
  return {
    city: loc.city,
    tempC: Math.round(cur.temperature_2m),
    feelsC: Math.round(
      typeof cur.apparent_temperature === "number"
        ? cur.apparent_temperature
        : cur.temperature_2m,
    ),
    humidity:
      typeof cur.relative_humidity_2m === "number" ? cur.relative_humidity_2m : 0,
    windKmh: Math.round(
      typeof cur.wind_speed_10m === "number" ? cur.wind_speed_10m : 0,
    ),
    code,
    label: meta.label,
    icon: meta.icon,
    updatedAt: Date.now(),
  };
}

export function useWeather() {
  const weather = ref<WeatherSnapshot | null>(null);
  const weatherError = ref("");
  const weatherLoading = ref(false);
  let timer: number | null = null;

  async function refreshWeather(quiet = false) {
    if (!quiet) weatherLoading.value = true;
    try {
      weather.value = await fetchWeather();
      weatherError.value = "";
    } catch (e) {
      weatherError.value = e instanceof Error ? e.message : String(e);
    } finally {
      weatherLoading.value = false;
    }
  }

  onMounted(() => {
    void refreshWeather();
    timer = window.setInterval(() => void refreshWeather(true), REFRESH_MS);
  });

  onUnmounted(() => {
    if (timer != null) window.clearInterval(timer);
  });

  return {
    weather,
    weatherError,
    weatherLoading,
    refreshWeather,
  };
}
