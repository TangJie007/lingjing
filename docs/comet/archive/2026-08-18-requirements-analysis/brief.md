# Outcome

Ship a runnable Windows desktop client (Tauri + Vue 3) that delivers the
LingJing MVP: an offline-first dynamic wallpaper experience covering PRD
functions F1–F9. The experience is pivoted on a three-zone immersive UI
(icon rail + wallpaper grid + right detail drawer + global playback bar),
with video/GIF wallpapers, local import/management, a self-built GPU-friendly
motion system, and performance/power auto-pause. The MVP is fully usable
offline with zero forced login.

# Scope

In scope (MVP, PRD F1–F9):

- F1 Wallpaper discovery & browsing: categorized grid, tag cloud, search, hover
  quick-actions, staggered card entrance.
- F2 Wallpaper detail & preview: right drawer (320px, frosted), large preview
  with Ken-Burns, 60s countdown ring, favorite/share/download/set actions,
  context menu.
- F3 Local resource management: drag/file/folder import of mp4/WebM/GIF/image,
  per-file progress, grid/list views, local-only favorites, library root path
  setting.
- F4 Wallpaper engine/player: video (MP4/WebM) and animated GIF/WebP playback;
  hardware-decode path planned; per-wallpaper-type performance tier.
- F5 Monitor assignment: architecture/hooks prepared in MVP; full multi-display
  feature lands in V1.0 (structural non-goal for MVP functional acceptance).
- F6 Playback control bar: global bottom bar with thumbnail, prev/next,
  play/pause, volume/mute, loop mode, quick import.
- F7 My / Favorites: local-only favorites, no login gate (login optional, V2).
- F8 Settings center: autostart, double-click hide, fullscreen-pause toggle,
  library path, global volume, UI click sound, theme switch.
- F9 Performance & power: fullscreen/game auto-pause, battery auto-pause,
  per-type performance tier, RDP pause, reduced-motion / low-end degradation.

# Non-goals

- Web/HTML5, Shader/GLSL, 3D wallpapers (V1.0 / V2 / V3).
- Multi-monitor assignment as a usable feature (architecture only in MVP).
- Light community / creative workshop, playlists/scheduling, editor, audio
  visualization, desktop widgets, cross-device sync (V1.0–V3).
- Cloud sync / mandatory login / account system.
- Online resource library backend: MVP uses bundled/local placeholder catalog;
  real online fetching is V1.0.
- Dreamy Aurora (二次元) skin beyond token scaffolding; Frost Light is the
  only fully specified theme baseline.

# Acceptance examples

A1. App launches on Windows 10/11, renders three-zone shell with icon rail
(80px), wallpaper grid, and (collapsible) right detail drawer; bottom global
playback bar present.

A2. User imports a local mp4/WebM/GIF via drag or file picker; file appears in
local library with progress; selecting it sets it as the live wallpaper and
plays with < 3% GPU under hardware decode.

A3. Hovering a card reveals a quick-action bar (▶ preview / set wallpaper);
clicking a card slides in the 320px frosted detail drawer with large preview
and a 60s countdown ring that auto-collapses at zero (cancellable).

A4. Favorites persist locally without login; "已收藏 N 张 · 本地保存" hint shows.

A5. Entering fullscreen app, switching to battery, or starting an RDP session
auto-pauses the wallpaper; resuming restores playback; `prefers-reduced-motion`
disables entrance/transition animations.

A6. Settings center toggles autostart, fullscreen-pause, UI click sound, and
theme (Frost Light baseline + dark token set); library root path is editable
and migrates files.

A7. Motion system uses only `transform`/`opacity`; duration tokens 150/280/520ms
and easing tokens (standard / spring) are defined as CSS variables and applied
consistently.

A8. Theme baseline is Frost Light (浅色): `--bg #F4F5F7`, `--surface #FFFFFF`,
`--primary #4F46E5`, `--secondary #0891B2`, `--text #1F2329`, `--text-2 #6B7280`,
`--border #E5E7EB`; switching to dark reuses the same token structure.

# Constraints and invariants

- Offline-first, zero forced login for all MVP core flows.
- Motion iron rule: animate only `transform` / `opacity` (GPU-friendly, no
  reflow).
- Theme tokenization: all colors/spacing/radius/shadow are `var(--xxx)`;
  switching theme replaces only a few root variables, zero structural change.
- Tauri window chrome must remain draggable: `app-region: drag` on the shell
  header, `no-drag` on interactive controls.
- WCAG AA contrast for Frost Light; full keyboard reachability; `reduced-motion`
  universal degradation.
- Install package target < 200MB; video wallpaper < 3% GPU (hardware decode);
  paused state ~0% occupancy.
- Current stack: Vue 3 + Vite + Tauri, Tailwind CSS v4 (`@tailwindcss/vite`),
  CSS-variable token theme already scaffolded in `src/styles/global.css`.

# Decisions

D1. Analysis scope = MVP only (F1–F9). Confirmed by user.
D2. Deliverable form = gap analysis + structured specification. Confirmed by
    user.
D3. Theme baseline = Frost Light (浅色) as primary, with dark token set as a
    switchable alternate; Dreamy Aurora deferred. Confirmed by user (PRD §8).
D4. Capability decomposition: MVP split into 7 specs — shell-layout,
    discover-browse (F1), detail-preview (F2), local-library (F3 + F7 local
    favorites), wallpaper-engine (F4), playback-control (F6), performance-power
    (F9); F5 multi-monitor and F8 settings are represented as architecture
    hooks / a settings spec respectively. Confirmed by agent.
D5. Online catalog is out of MVP backend scope; MVP uses a local placeholder
    catalog so F1 browsing is demonstrable offline. Confirmed by agent.

# Open questions

- [blocking] CONFIRM: MVP scope F1–F9, deliverable = gap analysis + specs,
  theme baseline = Frost Light, with the 7-capability decomposition above and
  local placeholder catalog for F1. Proceed to Design?

# Verification expectations

- Static review of brief + specs against PRD F1–F9 acceptance points.
- Confirm each acceptance item A1–A8 is covered by at least one capability spec.
- Confirm non-goals explicitly exclude V1.0+ functions and forced login.
- Confirm theme tokens in specs match PRD §8.2 Frost Light values.
- No code implementation is produced in this change; verification is
  specification-completeness only.
