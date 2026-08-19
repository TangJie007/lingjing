# Local Resource Management & Favorites (F3, F7)

This spec describes the complete local library experience after Archive,
including local-only favorites (F7). Login/cloud sync is V2 and out of MVP.

## Import

- Sources: drag-and-drop, file picker, folder batch import.
- Accepted formats: mp4 / WebM / GIF / image.
- Per-file progress bar with completion state; failed files are retryable.

## File list

- Grid / list dual view; filter by type / size / time.
- Same detail drawer as online catalog (see detail-preview).

## Local favorites (F7)

- Pure local favorites, no login gate (product differentiation).
- Persisted to local storage; shows "已收藏 N 张 · 本地保存" hint.
- Favorite grid present under "我的 / 收藏" rail entry.

## Storage path setting

- Custom wallpaper library root directory; migration / cleanup tooling.
- Editable in Settings (F8); changing path migrates existing files.

## Acceptance mapping

- Covers A2 (import → local library → set live) and A4 (local favorites
  persistence + hint).
