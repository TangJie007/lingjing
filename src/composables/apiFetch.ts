import { fetch as tauriFetch } from "@tauri-apps/plugin-http";

/**
 * Native HTTP via tauri-plugin-http — bypasses WebView CORS.
 * Falls back to window.fetch if the plugin call fails (API hosts usually allow CORS;
 * object-storage downloads must not rely on this fallback).
 */
export async function apiFetch(
  input: string,
  init?: RequestInit,
): Promise<Response> {
  try {
    return await tauriFetch(input, init);
  } catch (e) {
    if (typeof fetch === "function") {
      try {
        return await fetch(input, init);
      } catch {
        throw e instanceof Error ? e : new Error(String(e));
      }
    }
    throw e;
  }
}
