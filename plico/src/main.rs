//! Alternative frontend; the original Qt application remains independently buildable.
mod danmaku;
mod geometry;
mod overlay;
mod service;
mod ui;
use plico_renderer::{Point, PointerButton, PointerPhase};
use plico_shell::{ExtensionContext, ShellExtension, WakeCallback, WindowResourceKey};
use plico_vello::{RenderBackend, SurfaceSource, WindowLevel, WindowOptions};
use plico_video::{Player, PlayerOptions};
use snafu::Snafu;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[derive(usage_rs::Cli)]
#[usage(bin = "bilimini-plico", about = "Plico frontend for bili-mini")]
struct Cli {
    /// Start in the always-on-top compact player.
    #[usage(long)]
    mini: bool,
    /// Open a local file or URL without contacting the Bilibili API.
    #[usage(long)]
    video: Option<String>,
    /// Resolve and play a Bilibili BV identifier on startup.
    #[usage(long)]
    bvid: Option<String>,
    /// Stop automatically after this many seconds (manual smoke tests).
    #[usage(long)]
    quit_after: Option<u64>,
    /// Disable transparent rounded corners on surfaces without alpha support.
    #[usage(long)]
    opaque: bool,
    /// Use the null audio device for manual/headless verification.
    #[usage(long)]
    silent: bool,
}
#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("video service: {source}"))]
    Video { source: plico_video::Error },
    #[snafu(display("API thread: {source}"))]
    Api { source: std::io::Error },
    #[snafu(display("window host: {source}"))]
    Desktop { source: plico_vello::DesktopError },
}
struct WindowMode {
    mini: Arc<AtomicBool>,
    presented_mini: Arc<AtomicBool>,
    applied: Option<bool>,
    place_pending: bool,
    activity: Arc<overlay::OverlayActivity>,
    wake: Option<WakeCallback>,
    started: std::time::Instant,
    quit_after: Option<u64>,
}
impl ShellExtension for WindowMode {
    fn initialize(&mut self, wake: WakeCallback) -> Result<(), String> {
        self.wake = Some(wake);
        Ok(())
    }
    fn poll(&mut self, cx: &mut ExtensionContext<'_>) {
        let key = plico_shell::initial_window_resource_key(0);
        if cx.window_metrics(key).is_none() {
            return;
        }
        if self
            .quit_after
            .is_some_and(|s| self.started.elapsed().as_secs() >= s)
        {
            cx.exit();
            return;
        }
        let mini = self.mini.load(Ordering::Relaxed);
        let (w, h) = if mini {
            geometry::MINI_SIZE
        } else {
            (500, 760)
        };
        if self.applied == Some(mini) {
            if cx.window_metrics(key).is_some_and(|m| {
                m.logical_width.abs_diff(w) <= 1 && m.logical_height.abs_diff(h) <= 1
            }) {
                // Commit the new layout only after the asynchronous native resize.
                // Otherwise a compact tree is first drawn into the tall expanded
                // surface and the compositor stretches that frame during resize.
                if self.presented_mini.swap(mini, Ordering::Relaxed) != mini {
                    self.activity.poke(None);
                    if let Some(wake) = &self.wake {
                        wake();
                    }
                }
                if self.place_pending {
                    self.place_pending = false;
                    cx.position_window(key, i32::MAX, i32::MAX);
                }
            }
            return;
        }
        if cx.resize_window(key, w, h) {
            self.applied = Some(mini);
            // Native resize is asynchronous: place only once the new metrics
            // arrive, otherwise clamping still uses the old expanded size.
            self.place_pending = mini;
        }
    }
    fn window_metrics_changed(&mut self, key: WindowResourceKey, cx: &mut ExtensionContext<'_>) {
        if key == plico_shell::initial_window_resource_key(0) {
            // Commit at the resize event, even when idle/paused and no media
            // or service worker will emit another asynchronous wake.
            self.poll(cx);
        }
    }
    fn pointer_moved(&mut self, _: WindowResourceKey, point: Point, _: &mut ExtensionContext<'_>) {
        if self.mini.load(Ordering::Relaxed) {
            self.activity.poke(Some(point));
            if let Some(wake) = &self.wake {
                wake();
            }
        }
    }
    fn pointer_left(&mut self, _: WindowResourceKey, _: &mut ExtensionContext<'_>) {
        if self.mini.load(Ordering::Relaxed) {
            self.activity.poke(None);
            if let Some(wake) = &self.wake {
                wake();
            }
        }
    }
    fn pointer_button(
        &mut self,
        key: WindowResourceKey,
        button: PointerButton,
        phase: PointerPhase,
        point: Point,
        cx: &mut ExtensionContext<'_>,
    ) -> bool {
        if phase == PointerPhase::Down {
            self.activity.poke(Some(point));
            if let Some(wake) = &self.wake {
                wake();
            }
        }
        let metrics = cx.window_metrics(key);
        let width = metrics.map_or(500., |m| f64::from(m.logical_width));
        let height = metrics.map_or(760., |m| f64::from(m.logical_height));
        if button == PointerButton::Primary
            && phase == PointerPhase::Down
            && drag_area(
                point,
                width,
                height,
                self.mini.load(Ordering::Relaxed),
                self.activity.visible.load(Ordering::Relaxed),
            )
        {
            return cx.start_window_drag(key);
        }
        false
    }
}
fn main() -> Result<(), Error> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "warn".into()),
        )
        .try_init();
    let cli = Cli::parse();
    let mini = Arc::new(AtomicBool::new(cli.mini));
    let player = Player::with_options(PlayerOptions {
        width: 1280, height: 720, muted: cli.silent,
        referrer: Some("https://www.bilibili.com/".into()),
        user_agent: Some("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36".into()),
    }).map_err(|source| Error::Video { source })?;
    let surface = player.surface();
    let view = ui::App::new(player, mini.clone(), cli.video, cli.bvid)
        .map_err(|source| Error::Api { source })?;
    let activity = view.activity.clone();
    let presented_mini = view.presented_mini.clone();
    presented_mini.store(false, Ordering::Relaxed);
    let source = SurfaceSource::new(view, surface, "video");
    let renderer = if cfg!(feature = "skia") {
        RenderBackend::Skia
    } else {
        RenderBackend::Vello
    };
    eprintln!("bilimini-plico: renderer={renderer:?}");
    let options = WindowOptions::new()
        .title("bili-mini · Plico")
        .initial_inner_size(500, 760)
        .min_inner_size(320, 180)
        .max_inner_size(1280, 1440)
        .decorations(false)
        .corner_radius(if cli.opaque { 0 } else { 12 })
        .window_level(WindowLevel::AlwaysOnTop)
        .renderer(renderer);
    plico_shell::run_windows_with_factory_and_extensions(
        vec![(Box::new(source), options)],
        None,
        vec![Box::new(WindowMode {
            mini,
            presented_mini,
            applied: None,
            place_pending: false,
            activity,
            wake: None,
            started: std::time::Instant::now(),
            quit_after: cli.quit_after,
        })],
    )
    .map_err(|source| Error::Desktop { source })?;
    Ok(())
}

fn drag_area(point: Point, width: f64, height: f64, mini: bool, controls_visible: bool) -> bool {
    // Keep the outer 12 logical pixels for native resizing and controls outside
    // the drag region. In mini mode the picture itself acts as a move surface.
    if point.x < 12. || point.x >= width - 12. || point.y < 12. || point.y >= height - 12. {
        return false;
    }
    if mini {
        return !controls_visible || point.y < height - f64::from(geometry::OVERLAY_HEIGHT);
    }
    point.y < 46. && point.x < width - 260.
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dragging_excludes_resize_edges_and_controls_but_includes_mini_picture() {
        let at = |x, y, mini| drag_area(Point { x, y }, 500., 360., mini, true);
        assert!(at(100., 28., false));
        assert!(at(220., 28., false));
        assert!(!at(350., 28., false));
        assert!(at(250., 160., true));
        assert!(!at(250., 160., false));
        assert!(!at(250., 310., true));
        assert!(!at(3., 160., true));
        assert!(!at(100., 3., true));
    }
}
