use crate::service::{Event, Job, Service, Video};
use plico_core::{ImageAsset, prelude::*};
use plico_vello::{HostAction, View, WindowCommand};
use plico_video::{Player, Snapshot};
use serde_json::Value;
use std::{
    collections::{HashMap, VecDeque},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

const PLAY: SvgAsset = plico_core::svg_asset!("../../app/icons/play.svg");
const PAUSE: SvgAsset = plico_core::svg_asset!("../../app/icons/pause.svg");
const CLOSE: SvgAsset = plico_core::svg_asset!("../../app/icons/close.svg");
const NEXT: SvgAsset = plico_core::svg_asset!("../../app/icons/next.svg");
const EXPAND: SvgAsset = plico_core::svg_asset!("../../app/icons/chevron-up.svg");
const VOLUME: SvgAsset = plico_core::svg_asset!("../../app/icons/volume.svg");
const MUTE: SvgAsset = plico_core::svg_asset!("../../app/icons/mute.svg");
const SCRIM: SvgAsset = SvgAsset::new(
    r#"<svg xmlns="http://www.w3.org/2000/svg" width="480" height="68" viewBox="0 0 480 68" preserveAspectRatio="none"><defs><linearGradient id="g" x1="0" y1="0" x2="0" y2="1"><stop stop-color="black" stop-opacity="0"/><stop offset="1" stop-color="black" stop-opacity="0.85"/></linearGradient></defs><rect width="480" height="68" fill="url(#g)"/></svg>"#,
);
const DANMAKU: SvgAsset = plico_core::svg_asset!("../../app/icons/danmaku.svg");
pub struct App {
    pub player: Player,
    pub service: Service,
    pub mini: Arc<AtomicBool>,
    pub presented_mini: Arc<AtomicBool>,
    pub activity: Arc<crate::overlay::OverlayActivity>,
    pointer_revision: u64,
    rendered_revision: u64,
    last_activity: Option<std::time::Duration>,
    pub items: Vec<Video>,
    pub selected: Option<Video>,
    pub snapshot: Snapshot,
    danmaku: crate::danmaku::Overlay,
    danmaku_enabled: bool,
    media_clock: crate::danmaku::MediaClock,
    query: TextBuffer,
    search: bool,
    comments: bool,
    comment_rows: Vec<Value>,
    covers: HashMap<String, ImageAsset>,
    cover_order: VecDeque<String>,
    loading: bool,
    auth_busy: bool,
    auth_name: String,
    error: Option<String>,
    list_ticket: u64,
    play_ticket: u64,
    fresh: u32,
    clock_key: (u64, bool, bool, bool),
    action: Option<HostAction>,
}
impl App {
    pub fn new(
        player: Player,
        mini: Arc<AtomicBool>,
        local: Option<String>,
        bvid: Option<String>,
    ) -> Result<Self, std::io::Error> {
        let service = Service::new()?;
        let mut app = Self {
            player,
            service,
            presented_mini: Arc::new(AtomicBool::new(mini.load(Ordering::Relaxed))),
            mini,
            activity: Arc::new(crate::overlay::OverlayActivity::default()),
            pointer_revision: 0,
            rendered_revision: 0,
            last_activity: None,
            items: Vec::new(),
            selected: None,
            snapshot: Snapshot::default(),
            danmaku: Default::default(),
            danmaku_enabled: true,
            media_clock: Default::default(),
            query: TextBuffer::new(),
            search: false,
            comments: false,
            comment_rows: Vec::new(),
            covers: HashMap::new(),
            cover_order: VecDeque::new(),
            loading: false,
            auth_busy: false,
            auth_name: String::new(),
            error: None,
            list_ticket: 0,
            play_ticket: 0,
            fresh: 1,
            clock_key: (0, false, false, false),
            action: None,
        };
        if let Some(path) = local {
            app.selected = Some(Video {
                title: "本地播放验证".into(),
                ..Default::default()
            });
            if let Err(e) = app.player.load(path, None) {
                app.error = Some(e.to_string());
            }
        } else {
            app.request_list();
            app.submit(Job::Auth(false));
            if let Some(bvid) = bvid {
                app.select(Video {
                    title: bvid.clone(),
                    bvid,
                    ..Default::default()
                });
            }
        }
        Ok(app)
    }
    fn submit(&mut self, job: Job) {
        if let Err(error) = self.service.send(job) {
            self.error = Some(error);
            self.loading = false;
            self.auth_busy = false;
        }
    }
    fn request_list(&mut self) {
        self.list_ticket += 1;
        self.loading = true;
        self.error = None;
        self.comments = false;
        if self.search {
            let query = self.query.text().unwrap_or_default();
            if query.trim().is_empty() {
                self.loading = false;
                return;
            }
            self.submit(Job::Search(self.list_ticket, query));
        } else {
            self.submit(Job::Feed(self.list_ticket, self.fresh));
        }
    }
    fn select(&mut self, video: Video) {
        self.play_ticket += 1;
        self.danmaku = Default::default();
        self.media_clock = Default::default();
        self.comment_rows.clear();
        self.comments = false;
        self.error = None;
        self.submit(Job::Play(self.play_ticket, video.clone()));
        self.selected = Some(video);
        self.mini.store(true, Ordering::Relaxed);
        self.activity.poke(None);
    }
    fn toggle_mini(&mut self) {
        self.mini.fetch_xor(true, Ordering::Relaxed);
        self.activity.poke(None);
    }
    #[component]
    #[rustfmt::skip]
    fn mini_view(&mut self, cx: &mut UiContext) {
        let [width, height] = cx.viewport_size();
        let now = cx.now();
        let pointer = self.activity.snapshot();
        if self.last_activity.is_none() || pointer.revision != self.rendered_revision {
            self.last_activity = Some(now);
            self.rendered_revision = pointer.revision;
        }
        let hovered = pointer.position.is_some_and(|p| {
            p.x >= 0.
                && p.x < width as f64
                && p.y >= (height - crate::geometry::OVERLAY_HEIGHT) as f64
                && p.y < height as f64
        });
        let visible = crate::overlay::show_controls(now, self.last_activity.unwrap(), self.snapshot.paused,
            self.snapshot.loading || self.selected.is_none() || self.error.is_some(), hovered);
        self.activity.set_visible(visible);
        column(cx).key("video").width_percent(100.).height_percent(100.).background(Color::Literal(0x000000)).clip(true)
        + |cx|
        {
            self.danmaku_layer(cx, [width, height]);
            if self.selected.is_none() && button(cx).text("展开并选择视频").ghost().clicked() {
                self.toggle_mini();
            }
            column(cx).position(Position::Absolute).inset_left(0.).inset_right(0.).inset_bottom(0.)
                .height(crate::geometry::OVERLAY_HEIGHT).opacity(if visible { 1. } else { 0. }).disabled(!visible)
                .transition(150) + |cx|
            {
                column(cx).absolute(0., 0.).width_percent(100.).height(crate::geometry::OVERLAY_HEIGHT) + |cx| {
                    svg(cx, &SCRIM).size(width, crate::geometry::OVERLAY_HEIGHT);
                };
                column(cx).width_percent(100.).padding_left(10).padding_right(8).padding_top(20).gap(4) + |cx| {
                    row(cx).width_percent(100.).height(26.).gap(4).align(Align::Center) + |cx| {
                        if mini_icon(cx, if self.snapshot.paused { "播放" } else { "暂停" },
                            if self.snapshot.paused { &PLAY } else { &PAUSE }) {
                            self.toggle_pause();
                        }
                        text(
                            cx,
                            self.selected.as_ref().map_or("bili-mini", |v| v.title.as_str()),
                        ).width(0.).grow(1.).min_width(0.).font_size(12).ellipsis(true);
                        text(cx, time(self.snapshot.position)).font_size(12);
                        if mini_icon(cx, "静音", if self.snapshot.volume > 0. { &VOLUME } else { &MUTE }) {
                            let _ =
                                self.player.volume(if self.snapshot.volume > 0. { 0. } else { 1. });
                        }
                        if mini_icon(cx, if self.danmaku_enabled { "关闭弹幕" } else { "开启弹幕" }, &DANMAKU) {
                            self.danmaku_enabled = !self.danmaku_enabled;
                        }
                        if mini_icon(cx, "展开", &EXPAND) {
                            self.toggle_mini();
                        }
                        if mini_icon(cx, "关闭", &CLOSE) {
                            self.action =
                                Some(HostAction::Window(WindowCommand::Close));
                        }
                    };
                    let mut position = self.snapshot.position;
                    if slider(
                        cx,
                        "播放进度",
                        &mut position,
                        0. ..=self.snapshot.duration.max(1.),
                    ).width_percent(100.).min_height(12.).height(12.).response().changed
                        && let Err(e) = self.player.seek(position)
                    {
                        self.error = Some(e.to_string());
                    }
                };
            };
        };
    }
    #[component]
    #[rustfmt::skip]
    fn danmaku_layer(&mut self, cx: &mut UiContext, size: [f32; 2]) {
        if !self.danmaku_enabled {
            self.danmaku.clear_active();
            return;
        }
        let paused = self.snapshot.paused || self.snapshot.loading || self.snapshot.ended;
        let position = self.media_clock.position(cx.now(), self.snapshot.position, paused);
        self.danmaku.update(position, size);
        if !paused && !self.danmaku.active.is_empty() {
            cx.request_animation_frame();
        }
        column(cx).absolute(0., 0.).width(size[0]).height(size[1]).clip(true) + |cx| {
            for active in &self.danmaku.active {
                let item = &self.danmaku.items[active.item];
                let x = self.danmaku.x(*active, position);
                let y = active.lane as f32 * 22. + 5.;
                cx.keyed(active.item as u64, |cx| {
                    column(cx).absolute(x, y).width(active.width + 8.).height(22.) + |cx| {
                        text(cx, &item.text).position(Position::Absolute).inset_left(1.).inset_top(1.)
                            .font_size(crate::danmaku::FONT_SIZE).weight(600).color(Color::Literal(0x000000));
                        text(cx, &item.text).position(Position::Absolute).inset_left(0.).inset_top(0.)
                            .font_size(crate::danmaku::FONT_SIZE).weight(600)
                            .color(Color::Literal(item.color & 0xffffff));
                    };
                });
            }
        };
    }
    fn toggle_pause(&mut self) {
        if let Err(e) = self.player.pause(!self.snapshot.paused) {
            self.error = Some(e.to_string());
        }
    }
    fn next_video(&mut self) {
        if let Some(index) = self
            .selected
            .as_ref()
            .and_then(|selected| self.items.iter().position(|v| v.bvid == selected.bvid))
            && let Some(video) = self.items.get(index + 1).cloned()
        {
            self.select(video);
        }
    }
    #[component]
    fn controls(&mut self, cx: &mut UiContext) {
        let mut position = self.snapshot.position;
        if slider(
            cx,
            "播放进度",
            &mut position,
            0. ..=self.snapshot.duration.max(1.),
        )
        .width_percent(100.)
        .response()
        .changed
            && let Err(e) = self.player.seek(position)
        {
            self.error = Some(e.to_string());
        }
        row(cx)
            .width_percent(100.)
            .gap(8)
            .align(Align::Center)
            .children(|cx| {
                if icon_button(
                    cx,
                    if self.snapshot.paused {
                        "播放"
                    } else {
                        "暂停"
                    },
                    if self.snapshot.paused { &PLAY } else { &PAUSE },
                ) {
                    self.toggle_pause();
                }
                if icon_button(
                    cx,
                    if self.danmaku_enabled {
                        "关闭弹幕"
                    } else {
                        "开启弹幕"
                    },
                    &DANMAKU,
                ) {
                    self.danmaku_enabled = !self.danmaku_enabled;
                }
                if icon_button(cx, "下一个", &NEXT) {
                    self.next_video();
                }
                text!(
                    cx,
                    "{} / {}",
                    time(self.snapshot.position),
                    time(self.snapshot.duration)
                )
                .font_size(13)
                .color(Color::Muted);
                column(cx).grow(1.).children(|_| {});
                let mut volume = self.snapshot.volume;
                if slider(cx, "音量", &mut volume, 0. ..=1.)
                    .width(90.)
                    .response()
                    .changed
                    && let Err(e) = self.player.volume(volume)
                {
                    self.error = Some(e.to_string());
                }
            });
    }
    #[component]
    #[rustfmt::skip]
    fn body(&mut self, cx: &mut UiContext) {
        if self.selected.is_some() {
            let mini = self.presented_mini.load(Ordering::Relaxed);
            let [video_width, video_height] = crate::geometry::video_size(cx.viewport_size(), mini);
            column(cx).key("video").width(video_width).height(video_height).shrink(0.).align_self(Align::Center)
                .background(Color::Literal(0x0a0b0d)).radius(8.).clip(true) + |cx|
            {
                self.danmaku_layer(cx, [video_width, video_height]);
                if self.snapshot.loading {
                    text(cx, "正在缓冲…").color(Color::Muted);
                }
            };
            self.controls(cx);
            if !mini {
                text(cx, &self.selected.as_ref().unwrap().title).font_size(16).weight(600);
                row(cx).gap(8) + |cx| {
                    if button(cx).text("继续浏览").ghost().clicked() {
                        self.comments = false;
                    }
                    if button(cx).text("评论").ghost().clicked() {
                        self.comments = true;
                        if let Some(video) = &self.selected
                            && !video.bvid.is_empty()
                        {
                            self.submit(Job::Comments(self.play_ticket, video.bvid.clone()));
                        }
                    }
                    if button(cx).text("收起视频").ghost().clicked() {
                        let _ = self.player.pause(true);
                        self.selected = None;
                    }
                };
            }
        }
    }
    #[component]
    #[rustfmt::skip]
    fn browse(&mut self, cx: &mut UiContext) {
        row(cx).gap(6).align(Align::Center) + |cx| {
            for (is_search, label) in [(false, "推荐"), (true, "搜索")] {
                cx.keyed(label, |cx| {
                    if button(cx).text(label).ghost()
                        .background(if self.search == is_search { Color::Selection } else { Color::Surface })
                        .color(if self.search == is_search { Color::Accent } else { Color::Muted }).clicked()
                    {
                        self.search = is_search;
                        self.request_list();
                    }
                });
            }
            column(cx).grow(1.) + |_| {};
            if button(cx).text("换一批").ghost().clicked() {
                self.fresh += 1;
                self.request_list();
            }
        };
        if self.search {
            row(cx).gap(8) + |cx| {
                let submit = text_input(cx, "搜索视频", &self.query).placeholder("搜索视频、BV 号").width(0.).grow(1.)
                    .response().submitted;
                if button(cx).text("搜索").primary().clicked() || submit {
                    self.request_list();
                }
            };
        }
        if self.loading {
            text(cx, "正在加载…").color(Color::Muted);
        }
        let mut chosen = None;
        column(cx).height(0.).grow(1.).min_height(0.).width_percent(100.).scroll_y(true).gap(4) + |cx| {
            if self.comments {
                if self.comment_rows.is_empty() {
                    text(cx, "正在加载评论，或暂无评论").color(Color::Muted);
                }
                for (i, comment) in self.comment_rows.iter().enumerate() {
                    cx.keyed(i as u64, |cx| {
                        column(cx).padding(12).gap(6).background(Color::Panel).radius(8.) + |cx| {
                            text(cx,
                                comment["uname"].as_str().or_else(|| comment["name"].as_str()).unwrap_or("用户")).color(Color::Accent).font_size(13);
                            text(cx, comment["message"].as_str().unwrap_or("")).font_size(14);
                        };
                    });
                }
            } else {
                for video in &self.items {
                    cx.keyed(video.key(), |cx| {
                        if video_row(cx, video, self.covers.get(&video.cover),
                            self.selected.as_ref().is_some_and(|v| v.bvid == video.bvid)) {
                            chosen = Some(video.clone());
                        }
                    });
                }
                if !self.loading && self.items.is_empty() {
                    text(cx, "暂无视频，试试搜索或换一批").color(Color::Muted);
                }
            }
        };
        if let Some(video) = chosen {
            self.select(video);
        }
    }
}
impl View for App {
    fn theme(&self) -> Theme {
        Theme {
            foreground: 0xecedf0,
            surface: 0x15161a,
            panel: 0x1f2127,
            muted: 0x9ba1ac,
            accent: 0xfb7299,
            on_accent: 0xffffff,
            selection: 0x462630,
            border: 0x303238,
            control: 0x1b1d22,
            control_hover: 0x272a32,
            control_pressed: 0x333640,
            accent_hover: 0xff8baa,
            accent_pressed: 0xe65f86,
            focus: 0xfb7299,
            ..Theme::dark()
        }
    }
    fn set_wake_callback(&mut self, wake: plico_renderer::WakeCallback) {
        self.service.set_wake(wake);
    }
    fn take_host_action(&mut self) -> Option<HostAction> {
        self.action.take()
    }
    fn poll_reload(&mut self) -> bool {
        let revision = self.activity.snapshot().revision;
        let pointer_changed = revision != self.pointer_revision;
        self.pointer_revision = revision;
        let snapshot = self.player.snapshot();
        let key = (
            (snapshot.position * 10.) as u64,
            snapshot.paused,
            snapshot.loading,
            snapshot.error.is_some(),
        );
        let mut changed = pointer_changed || self.clock_key != key;
        self.clock_key = key;
        self.snapshot = snapshot;
        while let Some(event) = self.service.next() {
            changed = true;
            match event {
                Event::Items(ticket, result) if ticket == self.list_ticket => {
                    self.loading = false;
                    match result {
                        Ok(items) => {
                            self.items = items;
                        }
                        Err(error) => self.error = Some(error),
                    }
                }
                Event::Cover(url, image) => {
                    if !self.covers.contains_key(&url) {
                        self.cover_order.push_back(url.clone());
                    }
                    self.covers.insert(url, image);
                    while self.covers.len() > 64 {
                        if let Some(old) = self.cover_order.pop_front() {
                            self.covers.remove(&old);
                        }
                    }
                }
                Event::Stream(ticket, result) if ticket == self.play_ticket => match result {
                    Ok(stream) => {
                        let video = stream["video_url"].as_str().unwrap_or("");
                        let audio = stream["audio_url"]
                            .as_str()
                            .filter(|s| !s.is_empty())
                            .map(str::to_owned);
                        if let Err(e) = self.player.load(video, audio) {
                            self.error = Some(e.to_string());
                        }
                        let cid = stream["cid"].as_u64().unwrap_or(0);
                        if let Some(selected) = &mut self.selected {
                            selected.cid = cid;
                        }
                        if cid > 0 {
                            self.submit(Job::Danmaku(ticket, cid));
                        }
                    }
                    Err(error) => self.error = Some(error),
                },
                Event::Danmaku(ticket, result) if ticket == self.play_ticket => match result {
                    Ok(items) => self.danmaku.replace(items),
                    Err(error) => eprintln!("danmaku unavailable: {error}"),
                },
                Event::Comments(ticket, result) if ticket == self.play_ticket => match result {
                    Ok(value) => {
                        self.comment_rows = value["items"].as_array().cloned().unwrap_or_default()
                    }
                    Err(error) => self.error = Some(error),
                },
                Event::Auth(import, result) => {
                    if import {
                        self.auth_busy = false;
                    }
                    match result {
                        Ok(value) => {
                            self.auth_name = if value["logged_in"].as_bool().unwrap_or(false)
                                || value["ok"].as_bool().unwrap_or(false)
                            {
                                value["uname"].as_str().unwrap_or("已登录").to_owned()
                            } else {
                                String::new()
                            }
                        }
                        Err(error) if import => self.error = Some(error),
                        Err(_) => {}
                    }
                }
                _ => {}
            }
        }
        changed
    }
    fn on_unhandled_key(&mut self, key: &plico_vello::KeyEvent) -> bool {
        if key.phase != plico_core::KeyPhase::Down {
            return false;
        }
        match key.key.as_str() {
            " " | "Space" => self.toggle_pause(),
            "ArrowLeft" => {
                let _ = self.player.seek(self.snapshot.position - 5.);
            }
            "ArrowRight" => {
                let _ = self.player.seek(self.snapshot.position + 5.);
            }
            "ArrowUp" => {
                let _ = self.player.volume(self.snapshot.volume + 0.05);
            }
            "ArrowDown" => {
                let _ = self.player.volume(self.snapshot.volume - 0.05);
            }
            "Escape" => self.toggle_mini(),
            _ => return false,
        }
        true
    }
    #[component]
    #[rustfmt::skip]
    fn show(&mut self, cx: &mut UiContext) {
        let mini = self.presented_mini.load(Ordering::Relaxed);
        if mini {
            self.mini_view(cx);
            return;
        }
        column(cx).width_percent(100.).height_percent(100.).gap(10).padding(16).background(Color::Surface) + |cx| {
            row(cx).height(32.).shrink(0.).gap(8).align(Align::Center) + |cx| {
                text(cx, "B").padding(4).radius(5.).background(Color::Accent).color(Color::OnAccent)
                    .font_size(13).weight(700);
                text(cx, "bili-mini").font_size(14).weight(600);
                column(cx).grow(1.) + |_| {};
                if !mini
                    && button(cx).text(if self.auth_busy {
                            "导入中…"
                        } else if self.auth_name.is_empty() { "登录" } else { &self.auth_name }).ghost().clicked()
                    && !self.auth_busy
                {
                    self.auth_busy = true;
                    self.submit(Job::Auth(true));
                }
                if button(cx).text(if mini { "展开" } else { "小窗" }).ghost().clicked() {
                    self.toggle_mini();
                }
                if icon_button(cx, "关闭", &CLOSE) {
                    self.action = Some(HostAction::Window(WindowCommand::Close));
                }
            };
            if let Some(error) = self.error.as_ref().or(self.snapshot.error.as_ref()) {
                text(cx, error).font_size(13).color(Color::Accent);
            }
            self.body(cx);
            if !mini {
                self.browse(cx);
            } else if self.selected.is_none() {
                text(cx, "展开窗口，选择一个视频开始播放").color(Color::Muted);
            }
        };
    }
}
#[component]
fn mini_icon(cx: &mut UiContext, label: &str, asset: &SvgAsset) -> bool {
    button(cx)
        .aria_label(label)
        .ghost()
        .width(26.)
        .height(26.)
        .min_height(26.)
        .padding(4)
        .children(|cx| {
            svg(cx, asset).size(16., 16.);
        })
        .response
        .clicked()
}
#[component]
fn icon_button(cx: &mut UiContext, label: &str, asset: &SvgAsset) -> bool {
    button(cx)
        .aria_label(label)
        .ghost()
        .children(|cx| {
            svg(cx, asset).size(17., 17.);
        })
        .response
        .clicked()
}
#[component]
#[rustfmt::skip]
fn video_row(
    cx: &mut UiContext,
    video: &Video,
    cover: Option<&ImageAsset>,
    selected: bool,
) -> bool {
    let response = button_base(cx, &video.title).width_percent(100.).padding(8).gap(12).align(Align::Start)
        .selected(selected).background(if selected { Color::Selection } else { Color::Surface })
        .hover_background(Color::ControlHover).radius(8.) + |cx|
    {
        column(cx).width(144.).height(81.).shrink(0.).background(Color::Panel).radius(6.).clip(true) + |cx| {
            if let Some(cover) = cover {
                image(cx, cover).size(144., 81.);
            }
        };
        column(cx).width(0.).grow(1.).min_width(0.).gap(7) + |cx| {
            text(cx, &video.title).font_size(14).weight(550).height(38.);
            text!(cx, "{} · {} 播放", video.up, views(video.views)).font_size(12).color(Color::Muted)
                .ellipsis(true);
            text(cx, time(video.duration as f64)).font_size(12).color(Color::Muted);
        };
    };
    response.response.clicked()
}
fn time(seconds: f64) -> String {
    let s = seconds.max(0.) as u64;
    format!("{}:{:02}", s / 60, s % 60)
}
fn views(n: i64) -> String {
    if n >= 10000 {
        format!("{:.1} 万", n as f64 / 10000.)
    } else {
        n.to_string()
    }
}

impl Drop for App {
    fn drop(&mut self) {
        let s = self.player.snapshot();
        eprintln!(
            "bilimini-plico: items={} covers={} frames={} position={:.2} audio_channels={} error={}",
            self.items.len(),
            self.covers.len(),
            s.frames,
            s.position,
            s.audio_channels,
            s.error.as_deref().unwrap_or("none")
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn playback_controls_and_video_slot_have_unique_keys_in_both_modes() {
        let player = Player::with_options(plico_video::PlayerOptions {
            muted: true,
            ..Default::default()
        })
        .unwrap();
        let mini = Arc::new(AtomicBool::new(false));
        // A missing local file avoids network requests; declaration must be valid
        // independently of decoder readiness and even when a media load fails.
        let mut app = App::new(
            player,
            mini.clone(),
            Some("/nonexistent/plico-ui-probe.mp4".into()),
            None,
        )
        .unwrap();
        app.danmaku.replace(vec![crate::danmaku::Comment {
            time: 1.,
            text: "弹幕不会抢走按钮事件".into(),
            color: 0xffffff,
            mode: 1,
        }]);
        app.snapshot.position = 1.;
        app.snapshot.paused = true;
        let mut engine = plico_core::Engine::default();
        for compact in [false, true, false] {
            mini.store(compact, Ordering::Relaxed);
            app.presented_mini.store(compact, Ordering::Relaxed);
            engine
                .frame_declarations(|cx| app.show(cx))
                .expect("playback UI must not duplicate node identities");
        }
    }
    #[test]
    fn floating_expand_button_receives_clicks_over_video_surface() {
        use plico_renderer::{
            FrameSource, Modifiers, Point, PointerButton, PointerEvent, PointerPhase, SemanticRole,
            UiEvent,
        };
        let player = Player::with_options(plico_video::PlayerOptions {
            muted: true,
            ..Default::default()
        })
        .unwrap();
        let surface = player.surface();
        let mini = Arc::new(AtomicBool::new(true));
        let app = App::new(
            player,
            mini.clone(),
            Some("/nonexistent/plico-ui-probe.mp4".into()),
            None,
        )
        .unwrap();
        let mut source = plico_vello::SurfaceSource::new(app, surface, "video");
        source
            .source
            .view
            .danmaku
            .replace(vec![crate::danmaku::Comment {
                time: 0.,
                text: "视频上的弹幕".into(),
                color: 0xffffff,
                mode: 1,
            }]);
        source.set_semantics_enabled(true);
        let mut text = plico_renderer::TextContext::new();
        source.build_frame(&mut text, 480, 270);
        let semantics = source.semantic_snapshot().unwrap();
        let bounds = semantics
            .node_by_role(SemanticRole::Button, "展开", None)
            .unwrap()
            .bounds;
        let point = Point {
            x: f64::from((bounds[0] + bounds[2]) * 0.5),
            y: f64::from((bounds[1] + bounds[3]) * 0.5),
        };
        for phase in [PointerPhase::Down, PointerPhase::Up] {
            source.prepare_for_event(&mut text);
            source.handle_event(UiEvent::Pointer(PointerEvent {
                phase,
                position: point,
                button: Some(PointerButton::Primary),
                buttons: if phase == PointerPhase::Down { 1 } else { 0 },
                modifiers: Modifiers::empty(),
                properties: Default::default(),
            }));
        }
        source.build_frame(&mut text, 480, 270);
        assert!(
            !mini.load(Ordering::Relaxed),
            "overlay click must reach application"
        );
    }
    #[test]
    fn compact_video_layout_preserves_aspect_and_fills_default_width() {
        use plico_renderer::FrameSource;
        fn slot(
            engine: &plico_core::Engine,
            parent: plico_core::Entity,
        ) -> Option<plico_core::Entity> {
            engine.find(parent, "video").or_else(|| {
                engine
                    .node(parent)?
                    .children
                    .iter()
                    .find_map(|child| slot(engine, *child))
            })
        }
        let player = Player::with_options(plico_video::PlayerOptions {
            muted: true,
            ..Default::default()
        })
        .unwrap();
        let app = App::new(
            player,
            Arc::new(AtomicBool::new(true)),
            Some("/nonexistent/plico-ui-probe.mp4".into()),
            None,
        )
        .unwrap();
        let mut source = plico_vello::Source::new(app);
        // A requested expansion must not change layout before the host confirms
        // its asynchronous native resize. Keep painting the compact geometry.
        source.view.mini.store(false, Ordering::Relaxed);
        let mut text = plico_renderer::TextContext::new();
        for (width, height) in [
            crate::geometry::MINI_SIZE,
            (480, 360),
            (400, 280),
            (800, 500),
        ] {
            let nodes = source.build_frame(&mut text, width, height);
            let entity = slot(&source.engine, source.engine.root()).expect("video slot");
            let id = source.adapter.backend_id(entity).unwrap();
            let node = nodes.iter().find(|node| node.node_id == id).unwrap();
            let [w, h] = node.content_size;
            assert!(
                (w - width as f32).abs() < 1. && (h - height as f32).abs() < 1.,
                "video must fill compact window: {w}x{h} at {width}x{height}"
            );
        }
    }
}
