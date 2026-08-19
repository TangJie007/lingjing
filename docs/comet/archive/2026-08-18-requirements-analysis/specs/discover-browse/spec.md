# Wallpaper Discovery & Browsing (F1)

This spec describes the complete browsing experience after Archive. The MVP
uses a **local placeholder catalog** (bundled JSON of demo entries with
thumbnails) so browsing is demonstrable offline; a real online resource
library is V1.0 scope and out of MVP.

## Grid & catalog

- Categorized grid display with pagination / infinite scroll.
- Each card shows: thumbnail, title, file size, type tag (video / GIF / image).
- Skeleton placeholder while lazy-loading; staggered card entrance (see
  shell-layout motion system).

## Category tag cloud

- Tags: 全部 · 推荐 · 游戏 · 动漫 · 风景 · 动物 · 科技 · 趣味 · 人机交互.
- Sort: 最热 / 最新.
- Tags multi-selectable; selection state is memorized (URL/state) across
  navigation within session.

## Search

- Top-bar search box with hot-word hints, results view, and empty / error
  states.
- Focus animation; Enter triggers search; results are filterable.

## Hover preview state

- Hovering a card floats a quick-action bar: `▶ 预览` / `设为壁纸`.
- Quick "set wallpaper" triggers the engine (see wallpaper-engine) and a Toast.

## Acceptance mapping

- Contributes to A1 (grid in center zone) and A3 (hover quick-actions).
- Defines the catalog contract that `local-library` and `wallpaper-engine`
  consume.
