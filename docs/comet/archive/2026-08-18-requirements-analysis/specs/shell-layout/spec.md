# Shell Layout, Motion System & Theme Baseline (F5 architecture, F8 entry)

This spec describes the complete shell after Archive: the three-zone immersive
layout, the self-built GPU-friendly motion system, and the tokenized theme
architecture whose baseline is Frost Light (浅色). It also defines the
structural hooks for multi-monitor assignment (F5) and the entry surface for
the Settings center (F8).

## Layout

- Three-zone structure (PRD §7.1):
  - Icon rail (left, 80px): navigation (online / local / my / settings / about);
    selected state uses indigo `#4F46E5` highlight with a left indicator bar.
  - Center: top bar (search + theme) + category tag cloud + wallpaper card grid
    with hover quick-actions.
  - Right detail drawer (320px, frosted glass), slides in on card click,
    collapsible.
  - Global bottom playback control bar (PRD §7.1, F6).
- Tauri window chrome: header carries `app-region: drag`; interactive controls
  (buttons, inputs, drawer handles) carry `app-region: no-drag`.
- Responsive to high-DPI / ultra-wide; no layout break at 1280×720 minimum.

## Motion system (PRD §7.2)

- Tokens (CSS variables):
  - `--dur-fast: 150ms` (hover/press)
  - `--dur-base: 280ms` (drawer/panel)
  - `--dur-slow: 520ms` (entrance)
  - `--ease: cubic-bezier(.4,0,.2,1)` (standard)
  - `--ease-spring: cubic-bezier(.34,1.7,.5,1)` (elastic rebound)
- Iron rule: animate only `transform` / `opacity`; never trigger reflow.
- Behaviors:
  - Sidebar tap: ripple at point + icon elastic rebound + through-slide
    indicator bar.
  - Settings gear rotates; favorites heart has heartbeat animation.
  - Card entrance: staggered upward float (stagger) with lazy load + skeleton
    placeholder.
  - Drawer slide uses `--dur-base` + `--ease`.
- Accessibility: `role` / `aria-label` / keyboard reachable; `prefers-reduced-
  motion` universally disables entrance/transition animations (see F9).

## Theme architecture (PRD §8)

- All colors/spacing/radius/shadow are `var(--xxx)`; switching theme replaces
  only a few root variables, zero structural change.
- Frost Light (baseline, ✅ finalized) tokens:
  - `--bg: #F4F5F7`
  - `--surface: #FFFFFF`
  - `--primary: #4F46E5`
  - `--secondary: #0891B2`
  - `--text: #1F2329`
  - `--text-2: #6B7280`
  - `--border: #E5E7EB`
- Frost Dark (✅) reuses the same token structure with pressed-dark surfaces and
  `--primary: #818CF8`.
- Dreamy Aurora (🔜 exploration) is deferred; only token scaffolding may exist.
- Theme switch is exposed in Settings (F8) and the top bar.

## Settings entry (F8)

- Settings center reachable from the icon rail; contains: autostart, fullscreen-
  pause toggle, library path, global volume, UI click sound toggle, theme
  switch. Full behavior specified in the `local-library` / `performance-power`
  specs where state lives.

## Multi-monitor hook (F5, architecture only)

- Engine and playback interfaces must be designed so a display-assignment layer
  (Span / Per-display / duplicate, DPI-aware) can be added in V1.0 without
  restructuring the player. No functional multi-monitor UI in MVP.

## Acceptance mapping

- Covers A1 (three-zone shell), A7 (motion tokens), A8 (Frost Light tokens).
- Sets the stage for A2–A6 via child capabilities.
