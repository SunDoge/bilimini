//! Pointer activity crosses the host/UI boundary as a snapshot, not callbacks.
use plico_renderer::Point;
use std::{
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

#[derive(Clone, Copy, Default)]
pub struct PointerActivity {
    pub position: Option<Point>,
    pub revision: u64,
}
pub struct OverlayActivity {
    pointer: Mutex<PointerActivity>,
    pub visible: AtomicBool,
}
impl Default for OverlayActivity {
    fn default() -> Self {
        Self {
            pointer: Mutex::new(PointerActivity::default()),
            visible: AtomicBool::new(true),
        }
    }
}
impl OverlayActivity {
    pub fn poke(&self, position: Option<Point>) {
        let mut p = self.pointer.lock().unwrap();
        p.position = position;
        p.revision = p.revision.wrapping_add(1);
    }
    pub fn snapshot(&self) -> PointerActivity {
        *self.pointer.lock().unwrap()
    }
    pub fn set_visible(&self, visible: bool) {
        self.visible.store(visible, Ordering::Relaxed);
    }
}
pub fn show_controls(
    now: Duration,
    last_activity: Duration,
    paused: bool,
    busy: bool,
    hovered: bool,
) -> bool {
    paused || busy || hovered || now.saturating_sub(last_activity) < Duration::from_millis(2400)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clock_controls_expiry_pause_and_pointer_retention() {
        let last = Duration::from_secs(10);
        assert!(show_controls(
            Duration::from_secs(12),
            last,
            false,
            false,
            false
        ));
        assert!(!show_controls(
            Duration::from_millis(12400),
            last,
            false,
            false,
            false
        ));
        assert!(show_controls(
            Duration::from_secs(20),
            last,
            true,
            false,
            false
        ));
        assert!(show_controls(
            Duration::from_secs(20),
            last,
            false,
            false,
            true
        ));
        assert!(show_controls(
            Duration::from_secs(20),
            last,
            false,
            true,
            false
        ));
    }
}
