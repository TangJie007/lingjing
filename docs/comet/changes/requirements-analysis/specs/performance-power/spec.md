# Performance & Power Management (F9)

This spec describes the complete MVP performance/power auto-pause behavior
after Archive (PRD §6 F9).

## Auto-pause strategies

- Fullscreen / game auto-pause: detect fullscreen application → pause wallpaper
  (~0% occupancy).
- Battery mode: laptop switches to battery → auto-pause.
- RDP session: remote desktop session → auto-pause.
- Resuming from any paused state restores playback.

## Performance tier

- Per-type tier (video < web < shader < 3d); MVP applies video/GIF tiers to
  bound decode quality.

## Degradation

- `prefers-reduced-motion`: disable entrance/transition animations app-wide
  (see shell-layout motion system).
- Low-end machine: reduce resolution / quality tier.
- Missing/moved wallpaper file: graceful degradation, no crash (non-functional
  requirement §9 stability).

## Settings linkage (F8)

- Fullscreen-pause toggle lives in Settings; default on.

## Acceptance mapping

- Fully covers A5 (fullscreen / battery / weather auto-pause + restore,
  reduced-motion degradation).
