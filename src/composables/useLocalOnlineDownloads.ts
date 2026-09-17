import { computed, ref } from "vue";
import { listOnlineFileDownloads } from "./useEngine";

/** IDs present in `.onlinefile/list.json` with a valid local file. */
const localDownloadedIds = ref<Set<string>>(new Set());
let loadPromise: Promise<void> | null = null;

export async function refreshLocalOnlineDownloads() {
  if (loadPromise) return loadPromise;
  loadPromise = (async () => {
    try {
      const items = await listOnlineFileDownloads();
      localDownloadedIds.value = new Set(items.map((i) => String(i.id)));
    } catch (e) {
      console.warn("[onlinefile] list failed", e);
    } finally {
      loadPromise = null;
    }
  })();
  return loadPromise;
}

export function markLocalOnlineDownloaded(id: string) {
  const next = new Set(localDownloadedIds.value);
  next.add(String(id));
  localDownloadedIds.value = next;
}

export function isLocalOnlineDownloaded(id?: string | null) {
  if (!id) return false;
  return localDownloadedIds.value.has(String(id));
}

export function useLocalOnlineDownloads() {
  const ids = computed(() => localDownloadedIds.value);
  return {
    ids,
    isDownloaded: isLocalOnlineDownloaded,
    refresh: refreshLocalOnlineDownloads,
    markDownloaded: markLocalOnlineDownloaded,
  };
}
