## ADDED Requirements

### Requirement: Placeholder catalog browsing (Phase 1)
The app MUST render a categorized wallpaper grid from a local placeholder
catalog (bundled JSON of demo entries with thumbnails), so browsing is
demonstrable offline. Each card shows thumbnail, title, size, and type tag
(video / GIF / image). A category tag cloud (全部·推荐·游戏·动漫·风景·动物·
科技·趣味·人机交互) and sort (最热/最新) are selectable. Top-bar search box
with hot-word hints and results/empty states.

#### Acceptance Criteria
- WHEN the app loads the catalog THEN a grid of placeholder wallpaper cards
  renders with thumbnail, title, size, and type tag. (A1-grid)
- WHEN the user selects a category tag or sort THEN the grid filters accordingly
  and the selection persists within the session. (A1-filter)
- WHEN the user types in search THEN results/empty states display. (A1-search)

### Requirement: Hover quick-actions and detail drawer (Phase 1 shell)
Hovering a card MUST reveal a quick-action bar (▶ 预览 / 设为壁纸). Clicking a
card MUST slide in the 320px frosted right drawer (no route jump) with a large
preview, a 60-second SVG countdown ring (auto-collapses at zero, cancellable),
and an action area (favorite / share / download / set). Right-click context
menu (set / favorite / share / details).

#### Acceptance Criteria
- WHEN the user hovers a card THEN a quick-action bar (预览 / 设为壁纸) appears.
  (A3-hover)
- WHEN the user clicks a card THEN the 320px frosted drawer slides in with a
  large preview and a 60s countdown ring that auto-collapses at zero but is
  cancellable. (A3-drawer)
- WHEN the user right-clicks a card THEN a context menu (set / favorite / share
  / details) appears and is keyboard reachable. (A3-menu)
