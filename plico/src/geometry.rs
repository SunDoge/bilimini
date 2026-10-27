//! Compact mode is the picture itself. Overlay controls consume no layout height.
pub const MINI_SIZE: (u32, u32) = (480, 270);
pub const CONTENT_PADDING: f32 = 16.;
pub const VIDEO_ASPECT: f32 = 16. / 9.;
pub const OVERLAY_HEIGHT: f32 = 68.;

pub fn video_size(viewport: [f32; 2], mini: bool) -> [f32; 2] {
    let width = if mini {
        viewport[0].max(1.)
    } else {
        (viewport[0] - CONTENT_PADDING * 2.).max(1.)
    };
    let height = if mini {
        viewport[1].max(1.)
    } else {
        width / VIDEO_ASPECT
    };
    let h = height.min(width / VIDEO_ASPECT);
    [h * VIDEO_ASPECT, h]
}
