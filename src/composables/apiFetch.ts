import { fetch as tauriFetch } from "@tauri-apps/plugin-http";

/**
 * Native HTTP via tauri-plugin-http — bypasses WebView CORS.
 * Falls back to window.fetch only if the plugin is unavailable (e.g. plain browser preview).
 */
export async function apiFetch(
  input: string,
  init?: RequestInit,
): Promise<Response> {
  try {
    return await tauriFetch(input, init);
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    // Plugin not registered / not in Tauri shell — last-resort browser fetch.
    if (/plugin|not allowed|unknown|ipc|tauri/i.test(msg) && typeof fetch === "function") {
      return fetch(input, init);
    }
    throw e;
  }
}
