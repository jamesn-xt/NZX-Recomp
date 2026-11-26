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
    use std::sync::atomic::Ordering::Relaxed;
    mode().is_some() && !FINISHED.load(Relaxed)
}

#[cfg(windows)]
static FINISHED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[cfg(windows)]
pub(crate) enum Blocker {
    Graphics,
    Compute,
    Indirect,
    Texture,
    Transfer,
    Geometry,
}

#[cfg(windows)]
static BLOCKERS: [std::sync::atomic::AtomicU64; 6] =
    [const { std::sync::atomic::AtomicU64::new(0) }; 6];

#[cfg(windows)]
pub(crate) fn note(blocker: Blocker) {
    if enabled() {
        BLOCKERS[blocker as usize].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
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

    if !enabled() {
        return false;
    }
    let Some(enforce) = mode() else { return false };
    static STATE: Mutex<(Cover, u64, u64)> = Mutex::new((Cover {
        quiet: 0, started: false, finished: false,
    }, 0, 0));
    let mut state = STATE.lock().unwrap_or_else(|e| e.into_inner());
    let (cover, previous_draws, last_log) = &mut *state;
    if cover.finished {
        return false;
    }
    let drawn = crate::draw::drawn();
    let blockers = BLOCKERS.each_ref().map(|value| value.swap(0, Ordering::Relaxed));
    let [graphics, compute, indirect, texture, transfer, geometry] = blockers;
    let draws = drawn.saturating_sub(*previous_draws) + graphics;
    *previous_draws = drawn;
    let [queued, due, waiting, partial, _] =
        crate::draw::HEAP_TEXTURES_QUEUE.each_ref().map(|count| count.load(Ordering::Relaxed));
    let census_current = crate::draw::scene_texture_census_current();
    let was_started = cover.started;
    let ready = assets_ready(census_current, queued, blockers);
    let hold = cover.hold(draws, ready);
    if cover.finished {
        FINISHED.store(true, Ordering::Relaxed);
    }
    if cover.started && (!was_started || !hold || frame.saturating_sub(*last_log) >= 60) {
        *last_log = frame;
        tracing::info!(frame, draws, graphics, compute, indirect, texture, transfer, geometry,
            queued, due, waiting, partial, census_current, hold, enforce,
            "first scene presentation readiness");
    }
    enforce && hold
}

#[cfg(any(windows, test))]
fn assets_ready(census_current: bool, queued: u64, blockers: [u64; 6]) -> bool {
    census_current && queued == 0 && blockers == [0; 6]
}

#[cfg(test)]
mod tests {
    use super::{Cover, assets_ready};

    #[test]
    fn pending_work_in_any_render_path_holds_even_with_an_empty_texture_queue() {
        for slot in 0..6 {
            let mut blockers = [0; 6];
            blockers[slot] = 1;
            assert!(!assets_ready(true, 0, blockers));
        }
        assert!(!assets_ready(false, 0, [0; 6]));
        assert!(!assets_ready(true, 1, [0; 6]));
        assert!(assets_ready(true, 0, [0; 6]));
    }

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
