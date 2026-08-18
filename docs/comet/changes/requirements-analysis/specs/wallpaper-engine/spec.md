# Wallpaper Engine / Player (F4)

This spec describes the complete MVP wallpaper engine after Archive. MVP
supports video (MP4/WebM) and animated GIF/WebP only; web/HTML5, Shader/GLSL,
and 3D are later versions (PRD §6 F4).

## Supported types (MVP)

- Video: MP4 / WebM — hardware-decode path (MPV / LAV-class backend) targeted
  for < 3% GPU occupancy.
- Animated GIF / WebP — built-in decoder.

## Rendering

- Wallpaper rendered behind the desktop shell (Tauri native layer / worker
  window), independent of the Vue UI process.
- Engine interface designed display-assignment-ready (Span / Per-display) for
  V1.0 without restructure (see shell-layout F5 hook).

## Performance tier

- Per-wallpaper-type tier: video < web < shader < 3D (MVP uses video/GIF tiers;
  later tiers reserved). Tier drives decode quality / resolution.

## Acceptance mapping

- Contributes to A2 (set live wallpaper, plays with < 3% GPU under hardware
  decode).
- Engine is the handoff target of "set wallpaper" actions from F1/F2/F3.
