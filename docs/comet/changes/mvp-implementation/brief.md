# Outcome

Implement the LingJing MVP (PRD F1–F9) in the existing Tauri 2 + Vue 3 +
Vite + Tailwind v4 desktop app: a runnable dynamic-wallpaper client with a
three-zone immersive UI (icon rail + wallpaper grid + right detail drawer +
global playback bar), video/GIF wallpaper engine, local import/library,
favorites, settings, and performance/power auto-pause. Offline-first, no
forced login.

# Scope

MVP implementation of PRD F1–F9, decomposed from the archived
`requirements-analysis` change's seven capability specs:
shell-layout, discover-browse (F1), detail-preview (F2), local-library (F3+F7),
wallpaper-engine (F4), playback-control (F6), performance-power (F9).

# Non-goals

- Web/HTML5, Shader/GLSL, 3D wallpapers (V1.0 / V2 / V3).
- Functional multi-monitor assignment (architecture hook only in MVP; F5).
- Cloud sync / mandatory login / account system.
- Online resource library backend (MVP uses local placeholder catalog).
- Dreamy Aurora skin beyond token scaffolding.

# Acceptance examples

A1. App builds and launches on Windows; three-zone shell renders (icon rail
80px, wallpaper grid, collapsible 320px frosted drawer, bottom playback bar).
A2. User imports local mp4/WebM/GIF via drag or file picker; file appears in
local library with progress; selecting it sets it as the live wallpaper.
A3. Hovering a card reveals quick-actions; clicking a card slides in the 320px
frosted drawer with large preview + 60s countdown ring (cancellable).
A4. Favorites persist locally without login; "已收藏 N 张 · 本地保存" hint shows.
A5. Fullscreen / battery / RDP auto-pause; resume restores playback;
prefers-reduced-motion disables entrance/transition animations.
A6. Settings toggles autostart, fullscreen-pause, library path, volume, click
sound, theme (Frost Light baseline + dark).
A7. Motion system animates only transform/opacity with 150/280/520ms tokens.
A8. Frost Light theme tokens applied: bg #F4F5F7, surface #FFFFFF, primary
#4F46E5, secondary #0891B2, text #1F2329, text-2 #6B7280, border #E5E7EB.

# Constraints and invariants

- Offline-first, zero forced login for all MVP core flows.
- Motion iron rule: animate only transform/opacity.
- Theme tokenization: all colors/spacing/radius/shadow are var(--xxx).
- Tauri window chrome draggable: app-region drag on shell header, no-drag on
  controls.
- Existing stack: Vue 3 + Vite + Tauri 2.11, Tailwind CSS v4 already
  configured.
- Install < 200MB; video wallpaper < 3% GPU (hardware decode target); paused
  ~0% occupancy.

# Decisions

D1. Wallpaper engine architecture = dedicated Rust/Tauri worker window placed
    below the desktop icon layer via Win32 Progman/WorkerW, with the Vue main
    window as the UI control layer. Confirmed by user (recommended, standard
    dynamic-wallpaper approach; best performance/compatibility).
D2. Implementation cadence = phased progression, each phase independently
    verifiable:
    - Phase 1: three-zone UI skeleton + Frost Light (浅色) theme + motion
      tokens + placeholder-catalog browsing (F1 UI, F2 drawer shell, shell-
      layout, discover-browse with fake data).
    - Phase 2: wallpaper engine (F4) + local import/library (F3) + favorites
      (F7) + playback bar (F6).
    - Phase 3: performance/power auto-pause (F9) + settings center (F8).
    Confirmed by user (recommended; lowers risk).
D3. MVP scope = PRD F1–F9, decomposed into the seven capability specs from the
    archived requirements-analysis change. Confirmed earlier.
D4. Theme baseline = Frost Light (浅色); dark token set switchable. Confirmed
    earlier.

# Open questions

- [blocking] CONFIRM: MVP implementation of PRD F1–F9 using the phased plan
  above; engine = Win32 worker-window-below-desktop; theme baseline = Frost
  Light; cadence = Phase 1 (UI+theme+motion+fake-data) → Phase 2
  (engine+import+favorites+playback) → Phase 3 (performance+settings). Proceed
  to Build starting with Phase 1?

# Verification expectations

- Static + build verification: `pnpm build` (vue-tsc + vite build) passes.
- Tauri dev build launches; manual acceptance A1–A8 demonstrable.
- Frost Light tokens and motion tokens applied per spec.
- No forced login; offline flow works.
