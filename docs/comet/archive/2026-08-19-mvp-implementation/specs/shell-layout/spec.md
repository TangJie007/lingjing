## ADDED Requirements

### Requirement: Three-zone immersive shell
The app MUST render a three-zone immersive layout on launch:
- Left icon rail (80px) with navigation entries (online / local / my /
  settings) and a selected-state indicator.
- Center area: top bar (search + theme toggle) + category tag cloud +
  wallpaper card grid.
- Right detail drawer (320px, frosted glass), collapsible, slides in on card
  click.
- Global bottom playback control bar.
Tauri window header keeps `app-region: drag`; interactive controls keep
`app-region: no-drag`.

#### Acceptance Criteria
- WHEN the app launches THEN a three-zone shell renders with an 80px icon rail,
  a wallpaper grid, a collapsible 320px frosted drawer, and a bottom playback
  bar. (A1)
- WHEN the app launches THEN the shell header is draggable via `app-region:
  drag` while buttons/inputs remain interactive (`no-drag`). (A1-drag)

### Requirement: Frost Light theme baseline
The app MUST apply the Frost Light (浅色) token baseline:
`--bg #F4F5F7`, `--surface #FFFFFF`, `--primary #4F46E5`,
`--secondary #0891B2`, `--text #1F2329`, `--text-2 #6B7280`,
`--border #E5E7EB`. A dark token set MUST be switchable via the same token
structure. All colors/spacing/radius/shadow are `var(--xxx)`.

#### Acceptance Criteria
- WHEN the app renders in default theme THEN background uses `#F4F5F7`, surface
  `#FFFFFF`, primary `#4F46E5`, text `#1F2329`. (A8)
- WHEN the user toggles theme THEN only root token variables change; no
  structural change occurs. (A8-switch)

### Requirement: GPU-friendly motion system
The app MUST animate using only `transform` / `opacity` (no reflow). Motion
tokens: `--dur-fast 150ms`, `--dur-base 280ms`, `--dur-slow 520ms`,
`--ease cubic-bezier(.4,0,.2,1)`, `--ease-spring cubic-bezier(.34,1.7,.5,1)`.
Card entrance uses staggered upward float with skeleton placeholders.

#### Acceptance Criteria
- WHEN cards mount or the drawer opens THEN transitions use only transform/
  opacity and durations from the 150/280/520ms token set. (A7)
- WHEN `prefers-reduced-motion` is set THEN entrance/transition animations are
  disabled app-wide. (A7-reduced)
