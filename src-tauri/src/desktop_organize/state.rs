use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

pub(crate) const FENCE_LABEL: &str = "desktop-fence";
pub(crate) static ACTIVE: AtomicBool = AtomicBool::new(false);
pub(crate) static WATCH_GEN: AtomicU64 = AtomicU64::new(0);

pub(crate) fn is_active() -> bool {
    ACTIVE.load(Ordering::SeqCst)
}

pub(crate) fn set_active(active: bool) {
    ACTIVE.store(active, Ordering::SeqCst);
}

pub(crate) fn bump_watch_generation() -> u64 {
    WATCH_GEN.fetch_add(1, Ordering::SeqCst) + 1
}

pub(crate) fn watch_generation() -> u64 {
    WATCH_GEN.load(Ordering::SeqCst)
}
