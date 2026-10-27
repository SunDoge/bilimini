//! Bounded, deterministic scrolling overlay; playback time is supplied by the host.
use serde::Deserialize;
use std::time::Duration;

pub const FONT_SIZE: u16 = 14;
const MAX_ACTIVE: usize = 64;
#[derive(Debug, Deserialize)]
pub struct Comment {
    pub time: f64,
    pub text: String,
    pub color: u32,
    pub mode: u8,
}
#[derive(Clone, Copy)]
pub struct Active {
    pub item: usize,
    pub lane: usize,
    pub start: f64,
    pub width: f32,
}
#[derive(Default)]
pub struct Overlay {
    pub items: Vec<Comment>,
    pub active: Vec<Active>,
    next: usize,
    last: Option<f64>,
    size: [f32; 2],
    free: [f64; 9],
}
impl Overlay {
    pub fn replace(&mut self, mut items: Vec<Comment>) {
        items.retain(|c| {
            c.time.is_finite() && c.time >= 0. && !c.text.is_empty() && (1..=3).contains(&c.mode)
        });
        for c in &mut items {
            c.text = c.text.chars().take(160).collect();
        }
        items.sort_by(|a, b| a.time.total_cmp(&b.time));
        items.truncate(20000);
        *self = Self {
            items,
            ..Self::default()
        };
    }
    pub fn clear_active(&mut self) {
        self.active.clear();
        self.last = None;
    }
    pub fn speed(&self) -> f32 {
        (self.size[0] + 180.) / 8.
    }
    pub fn x(&self, a: Active, now: f64) -> f32 {
        self.size[0] - (now - a.start).max(0.) as f32 * self.speed()
    }
    pub fn update(&mut self, now: f64, size: [f32; 2]) {
        if !now.is_finite() || size[0] < 40. || size[1] < 20. {
            return;
        }
        if self.size != size
            || self
                .last
                .is_none_or(|last| now < last - 0.1 || now > last + 1.)
        {
            self.active.clear();
            self.free = [f64::NEG_INFINITY; 9];
            self.next = self.items.partition_point(|c| c.time < now - 0.2);
            self.size = size;
        }
        self.last = Some(now);
        let speed = self.speed();
        self.active
            .retain(|a| size[0] - (now - a.start) as f32 * speed + a.width > 0.);
        let lanes = ((size[1] * 0.65 / 22.).floor() as usize).clamp(1, 9);
        while self.next < self.items.len() && self.items[self.next].time <= now {
            let item = self.next;
            self.next += 1;
            if self.items[item].time < now - 0.4 || self.active.len() >= MAX_ACTIVE {
                continue;
            }
            if let Some(lane) = self.free[..lanes].iter().position(|free| *free <= now) {
                // Conservative advance estimate. Equal speeds prevent later text
                // catching an earlier comment; busy lanes drop excess comments.
                let width = self.items[item]
                    .text
                    .chars()
                    .map(|c| if c.is_ascii() { 16. } else { 28. })
                    .sum::<f32>();
                self.free[lane] = now + f64::from((width + 30.) / speed);
                self.active.push(Active {
                    item,
                    lane,
                    start: now,
                    width,
                });
            }
        }
    }
}
#[derive(Default)]
pub struct MediaClock {
    anchor: Option<(Duration, f64)>,
    sample: f64,
    paused: bool,
}
impl MediaClock {
    pub fn position(&mut self, now: Duration, sample: f64, paused: bool) -> f64 {
        let estimate = self.anchor.map_or(sample, |(time, p)| {
            p + now.saturating_sub(time).as_secs_f64()
        });
        if self.anchor.is_none()
            || paused != self.paused
            || (sample != self.sample && (sample - estimate).abs() > 0.25)
        {
            self.anchor = Some((now, sample));
        }
        self.sample = sample;
        self.paused = paused;
        if paused {
            sample
        } else {
            self.anchor.map_or(sample, |(time, p)| {
                p + now.saturating_sub(time).as_secs_f64()
            })
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn comment(time: f64, text: &str) -> Comment {
        Comment {
            time,
            text: text.into(),
            color: 0xffffff,
            mode: 1,
        }
    }
    #[test]
    fn seek_resize_and_density_are_bounded() {
        let mut overlay = Overlay::default();
        overlay.replace(
            (0..200)
                .map(|_| comment(1., "测试弹幕"))
                .chain([comment(20., "after seek")])
                .collect(),
        );
        overlay.update(1., [480., 270.]);
        assert_eq!(overlay.active.len(), 7);
        let x = overlay.x(overlay.active[0], 1.);
        overlay.update(1.5, [480., 270.]);
        assert!(overlay.x(overlay.active[0], 1.5) < x);
        overlay.update(20., [480., 270.]);
        assert_eq!(overlay.active.len(), 1);
        assert_eq!(overlay.items[overlay.active[0].item].text, "after seek");
        overlay.update(1., [640., 360.]);
        assert_eq!(overlay.active.len(), 9);
    }
    #[test]
    fn clock_interpolates_and_freezes_on_pause_then_resyncs_seek() {
        let mut clock = MediaClock::default();
        assert_eq!(clock.position(Duration::ZERO, 10., false), 10.);
        assert_eq!(clock.position(Duration::from_millis(50), 10., false), 10.05);
        assert_eq!(clock.position(Duration::from_secs(1), 10.1, true), 10.1);
        assert_eq!(clock.position(Duration::from_secs(2), 10.1, true), 10.1);
        assert_eq!(clock.position(Duration::from_secs(3), 30., false), 30.);
    }
}
