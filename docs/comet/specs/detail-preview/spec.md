# Wallpaper Detail & Preview (F2)

This spec describes the complete right-detail-drawer experience after Archive.

## Right detail drawer

- Slides in from the right on card click (no route jump, PRD §6 F2); width
  320px; frosted-glass texture; collapsible.
- Uses `--dur-base` + `--ease` from the motion system.

## Large preview

- Large preview image/video inside the drawer with Ken-Burns easing; animation
  uses only `transform` / `opacity`.

## 60-second countdown ring

- SVG ring progress; counts down from 60s; at zero auto-collapses the drawer
  (飞火 paradigm). Real decrement; user can manually cancel before zero.

## Action area

- Favorite / Share / Download / Set as wallpaper.
- Favorite heart has heartbeat animation.
- "Set as wallpaper" triggers a Toast and hands off to the engine
  (wallpaper-engine).

## Context menu

- Right-click context menu: set / favorite / share / details; keyboard
  reachable.

## Acceptance mapping

- Fully covers A3 (drawer, large preview, 60s ring, actions, context menu).
