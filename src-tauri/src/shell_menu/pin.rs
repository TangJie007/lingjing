use windows::Win32::UI::Shell::IContextMenu;

use crate::desktop_organize::ShellMenuEntry;

use super::entry::sep;
use super::verbs::command_verb;

pub(crate) fn pin_icon_svg(kind: &str) -> String {
    // Fallback glyphs; frontend prefers `src/assets/svg/*` when rendering.
    let path = match kind {
        "cut" => "M14 4l-4 4 4 4M6 4v12M10 8H2",
        "copy" => "M6 6h8v10H6zM4 4h8",
        "rename" => "M3 13l7-7 3 3-7 7H3v-3zM11 5l2 2",
        "share" => "M12 4v3c-5 0-8 2-9 6 2-2 4-3 9-3v3l5-4.5L12 4z",
        "delete" => "M5 6h10M7 6V5h6v1M7 8v7h6V8",
        "refresh" => "M12 3a7 7 0 0 1 7 7h-2a5 5 0 1 0-1.5 3.5L17 12v4h-4l1.2-1.2A7 7 0 1 1 12 3z",
        _ => "M4 8h12",
    };
    let svg = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' width='16' height='16' viewBox='0 0 16 16' fill='none' stroke='%23f3f3f3' stroke-width='1.4' stroke-linecap='round' stroke-linejoin='round'><path d='{path}'/></svg>"
    );
    format!(
        "data:image/svg+xml;utf8,{}",
        svg.replace('#', "%23").replace('\'', "%27")
    )
}

pub(crate) fn pin_kind_from_verb_or_label(verb: &str, label: &str) -> Option<&'static str> {
    let v = verb.to_ascii_lowercase();
    if matches!(v.as_str(), "cut") || label.contains("剪切") {
        return Some("cut");
    }
    if matches!(v.as_str(), "copy") || label.contains("复制") {
        return Some("copy");
    }
    if matches!(v.as_str(), "rename") || label.contains("重命名") {
        return Some("rename");
    }
    if matches!(v.as_str(), "delete") || label.contains("删除") {
        return Some("delete");
    }
    if v.contains("share") || label.contains("共享") || label.contains("分享") {
        return Some("share");
    }
    None
}

/// Pull Win11 common actions into a pinned top strip; keep the rest below.
pub(crate) fn apply_win11_pin_row(pcm: Option<&IContextMenu>, entries: Vec<ShellMenuEntry>) -> Vec<ShellMenuEntry> {
    use super::entry::item;
    use super::ids::BUILTIN_RENAME;

    const ORDER: &[&str] = &["cut", "copy", "rename", "share", "delete"];
    let mut slots: [Option<ShellMenuEntry>; 5] = [None, None, None, None, None];
    let mut taken = std::collections::HashSet::<u32>::new();

    for entry in &entries {
        if entry.separator || entry.children.is_some() || entry.disabled || entry.id == 0 {
            continue;
        }
        let verb = pcm
            .and_then(|p| command_verb(p, entry.id))
            .unwrap_or_default();
        let Some(kind) = pin_kind_from_verb_or_label(&verb, &entry.label) else {
            continue;
        };
        let Some(idx) = ORDER.iter().position(|k| *k == kind) else {
            continue;
        };
        if slots[idx].is_some() {
            continue;
        }
        let mut pinned = entry.clone();
        pinned.pin = true;
        // Always use our pin glyphs (frontend may replace with asset SVGs).
        pinned.icon = Some(pin_icon_svg(kind));
        pinned.destructive = kind == "delete";
        // Delete always goes through our Recycle Bin path (SHFileOperation).
        if kind == "delete" {
            use super::ids::BUILTIN_DELETE;
            pinned.id = BUILTIN_DELETE;
        }
        pinned.label = match kind {
            "cut" => "剪切".into(),
            "copy" => "复制".into(),
            "rename" => "重命名".into(),
            "share" => "共享".into(),
            "delete" => "删除".into(),
            _ => pinned.label,
        };
        slots[idx] = Some(pinned);
        taken.insert(entry.id);
    }

    // Always keep 重命名 in the pin strip when any file action is present.
    let rename_idx = 2;
    let has_file_pins = slots.iter().enumerate().any(|(i, s)| i != rename_idx && s.is_some());
    if has_file_pins && slots[rename_idx].is_none() {
        let mut rename = item(BUILTIN_RENAME, "重命名");
        rename.pin = true;
        rename.icon = Some(pin_icon_svg("rename"));
        slots[rename_idx] = Some(rename);
    }

    let mut out = Vec::with_capacity(entries.len() + 2);
    let mut any_pin = false;
    for slot in slots {
        if let Some(p) = slot {
            any_pin = true;
            out.push(p);
        }
    }
    if any_pin {
        out.push(sep());
    }

    let mut last_was_sep = any_pin;
    for entry in entries {
        if taken.contains(&entry.id) && !entry.separator && entry.children.is_none() {
            continue;
        }
        // Drop body duplicates of pinned actions (including forced rename).
        if !entry.separator && entry.children.is_none() && any_pin {
            if let Some(kind) = pin_kind_from_verb_or_label("", &entry.label) {
                if ORDER.contains(&kind) {
                    continue;
                }
            }
        }
        if entry.separator {
            if last_was_sep {
                continue;
            }
            last_was_sep = true;
            out.push(entry);
            continue;
        }
        last_was_sep = false;
        out.push(entry);
    }
    out
}

