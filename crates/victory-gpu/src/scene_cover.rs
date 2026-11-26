//! Optional presentation hold during the first heavy scene. Guest presents
//! and rendering continue, including diagnostic captures of covered frames.

#[derive(Default)]
struct Cover {
    quiet: u8,
    started: bool,
    finished: bool,
}

impl Cover {
    fn hold(&mut self, draws: u64, ready: bool) -> bool {
        const SCENE_DRAWS: u64 = 500;
        const READY_FRAMES: u8 = 3;
        if self.finished || (!self.started && draws < SCENE_DRAWS) {
            return false;
        }
        self.started = true;
        self.quiet = if ready && draws >= SCENE_DRAWS { self.quiet + 1 } else { 0 };
        self.finished = self.quiet == READY_FRAMES;
        !self.finished
    }
}

#[cfg(windows)]
pub(crate) fn enabled() -> bool {
    mode().is_some()
}

#[cfg(windows)]
fn mode() -> Option<bool> {
    static MODE: std::sync::OnceLock<Option<bool>> = std::sync::OnceLock::new();
    *MODE.get_or_init(|| match std::env::var("VICTORY_SCENE_COVER").as_deref() {
        Ok("1") => Some(true),
        Ok("observe") => Some(false),
        _ => None,
    })
}

#[cfg(windows)]
pub(crate) fn hold(frame: u64) -> bool {
    use std::sync::{Mutex, atomic::Ordering};

    let Some(enforce) = mode() else { return false };
    static STATE: Mutex<(Cover, u64, u64)> = Mutex::new((Cover {
        quiet: 0, started: false, finished: false,
    }, 0, 0));
    let mut state = STATE.lock().unwrap_or_else(|e| e.into_inner());
    let (cover, previous_draws, last_log) = &mut *state;
    if cover.finished {
        return false;
    }
    let (drawn, dropped) = crate::draw::scene_draw_counts();
    let draws = drawn.saturating_sub(*previous_draws) + dropped;
    *previous_draws = drawn;
    let [queued, due, waiting, partial, _] =
        crate::draw::HEAP_TEXTURES_QUEUE.each_ref().map(|count| count.load(Ordering::Relaxed));
    let census_current = crate::draw::scene_texture_census_current();
    let was_started = cover.started;
    let hold = cover.hold(draws, census_current && dropped == 0 && queued == 0);
    if cover.started && (!was_started || !hold || frame.saturating_sub(*last_log) >= 60) {
        *last_log = frame;
        tracing::info!(frame, draws, dropped, queued, due, waiting, partial, census_current, hold, enforce,
            "first scene presentation readiness");
    }
    enforce && hold
}

#[cfg(test)]
mod tests {
    use super::Cover;

    #[test]
    fn holds_the_first_heavy_scene_through_incomplete_frames() {
        let mut cover = Cover::default();
        assert!(!cover.hold(12, false));
        assert!(cover.hold(500, false));
        assert!(cover.hold(1000, true));
        assert!(cover.hold(1000, false));
        assert!(cover.hold(1000, true));
        assert!(cover.hold(10, true));
        assert!(cover.hold(1000, true));
        assert!(cover.hold(1000, true));
        assert!(!cover.hold(1000, true));
        assert!(!cover.hold(1000, false));
    }
}
