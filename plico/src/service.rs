//! Host actor: UI sends commands, receives immutable snapshots; no work in show.
use bili_core::bili::Bili;
use plico_core::ImageAsset;
use plico_renderer::WakeCallback;
use serde::Deserialize;
use serde_json::Value;
use std::{
    sync::{Arc, Mutex},
    thread::JoinHandle,
};
use tokio::{sync::mpsc, task::JoinSet};

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Video {
    pub bvid: String,
    pub cid: u64,
    pub title: String,
    pub up: String,
    pub duration: i64,
    pub views: i64,
    pub cover: String,
}
impl Video {
    pub fn key(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hash = std::collections::hash_map::DefaultHasher::new();
        self.bvid.hash(&mut hash);
        hash.finish()
    }
}
pub enum Job {
    Feed(u64, u32),
    Search(u64, String),
    Play(u64, Video),
    Comments(u64, String),
    Danmaku(u64, u64),
    Auth(bool),
}
pub enum Event {
    Items(u64, Result<Vec<Video>, String>),
    Cover(String, ImageAsset),
    Stream(u64, Result<Value, String>),
    Comments(u64, Result<Value, String>),
    Danmaku(u64, Result<Vec<crate::danmaku::Comment>, String>),
    Auth(bool, Result<Value, String>),
}
pub struct Service {
    tx: Option<mpsc::Sender<Job>>,
    rx: mpsc::Receiver<Event>,
    wake: Arc<Mutex<WakeCallback>>,
    worker: Option<JoinHandle<()>>,
}
impl Service {
    pub fn new() -> std::io::Result<Self> {
        let (tx, mut jobs) = mpsc::channel(16);
        let (events, rx) = mpsc::channel(32);
        let wake: Arc<Mutex<WakeCallback>> = Arc::new(Mutex::new(Arc::new(|| {})));
        let notifier = wake.clone();
        let worker = std::thread::Builder::new().name("bili-api".into()).spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("API runtime");
            runtime.block_on(async move {
                let api = Arc::new(Bili::new());
                let http = reqwest::Client::builder().timeout(std::time::Duration::from_secs(12))
                    .no_proxy().user_agent("Mozilla/5.0").build().expect("image HTTP client");
                let mut tasks = JoinSet::new();
                let mut list_task: Option<tokio::task::AbortHandle> = None;
                let mut play_task: Option<tokio::task::AbortHandle> = None;
                let mut danmaku_task: Option<tokio::task::AbortHandle> = None;
                loop {
                    tokio::select! {
                        job = jobs.recv() => {
                            let Some(job) = job else { break };
                            let api = api.clone();
                            let http = http.clone();
                            let events = events.clone();
                            let wake = notifier.clone();
                            let list = matches!(job, Job::Feed(..) | Job::Search(..));
                            let play = matches!(job, Job::Play(..));
                            let danmaku = matches!(job, Job::Danmaku(..));
                            if (play || danmaku) && let Some(task) = danmaku_task.take() { task.abort(); }
                            if list && let Some(task) = list_task.take() { task.abort(); }
                            if play && let Some(task) = play_task.take() { task.abort(); }
                            let task = tasks.spawn(async move {
                                let send = |event| async {
                                    if events.send(event).await.is_ok() {
                                        let callback = wake.lock().unwrap().clone();
                                        callback();
                                    }
                                };
                                match job {
                                    Job::Feed(ticket, page) => {
                                        let result = api.feed(page).await.map_err(|e| format!("{e:#}"));
                                        publish_items(ticket, result, &http, &events, &wake).await;
                                    }
                                    Job::Search(ticket, query) => {
                                        let result = api.search(&query, "video").await.map_err(|e| format!("{e:#}"));
                                        publish_items(ticket, result, &http, &events, &wake).await;
                                    }
                                    Job::Play(ticket, video) => {
                                        send(Event::Stream(ticket, api.play(&video.bvid, video.cid, 80).await.map_err(|e| format!("{e:#}")))).await;
                                    }
                                    Job::Danmaku(ticket, cid) => {
                                        let result = api.danmaku(cid).await.map_err(|e| format!("{e:#}"))
                                            .and_then(|values| values.into_iter().map(serde_json::from_value).collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string()));
                                        send(Event::Danmaku(ticket, result)).await;
                                    }
                                    Job::Comments(ticket, bvid) => {
                                        send(Event::Comments(ticket, api.comments(&bvid, "", "hot").await.map_err(|e| format!("{e:#}")))).await;
                                    }
                                    Job::Auth(import) => {
                                        let result = if import { api.import_cookies().await } else { api.auth_status().await };
                                        send(Event::Auth(import, result.map_err(|e| format!("{e:#}")))).await;
                                    }
                                }
                            });
                            if list { list_task = Some(task); } else if play { play_task = Some(task); } else if danmaku { danmaku_task = Some(task); }
                        }
                        _ = tasks.join_next(), if !tasks.is_empty() => {}
                    }
                }
                tasks.abort_all();
                while tasks.join_next().await.is_some() {}
            });
        })?;
        Ok(Self {
            tx: Some(tx),
            rx,
            wake,
            worker: Some(worker),
        })
    }
    pub fn send(&self, job: Job) -> Result<(), String> {
        self.tx
            .as_ref()
            .ok_or("API worker stopped")?
            .try_send(job)
            .map_err(|e| format!("API queue: {e}"))
    }
    pub fn next(&mut self) -> Option<Event> {
        self.rx.try_recv().ok()
    }
    pub fn set_wake(&self, wake: WakeCallback) {
        *self.wake.lock().unwrap() = wake;
    }
}
impl Drop for Service {
    fn drop(&mut self) {
        self.tx.take();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
async fn publish_items(
    ticket: u64,
    result: Result<Vec<Value>, String>,
    http: &reqwest::Client,
    events: &mpsc::Sender<Event>,
    wake: &Arc<Mutex<WakeCallback>>,
) {
    let result = result.and_then(|items| {
        items
            .into_iter()
            .map(serde_json::from_value)
            .collect::<Result<Vec<Video>, _>>()
            .map_err(|e| e.to_string())
    });
    let videos = result.as_ref().ok().cloned().unwrap_or_default();
    if events.send(Event::Items(ticket, result)).await.is_err() {
        return;
    }
    let callback = wake.lock().unwrap().clone();
    callback();
    // Four bounded download tasks; cancellation drops the set and its requests.
    let mut covers = JoinSet::new();
    for video in videos.iter().filter(|v| !v.cover.is_empty()).take(60) {
        if covers.len() >= 4 {
            let _ = covers.join_next().await;
        }
        let url = video.cover.clone();
        let http = http.clone();
        let events = events.clone();
        let ready = wake.clone();
        covers.spawn(async move {
            let response = http
                .get(&url)
                .header("Referer", "https://www.bilibili.com/")
                .send()
                .await
                .ok()?
                .error_for_status()
                .ok()?;
            if response
                .content_length()
                .is_some_and(|n| n > 4 * 1024 * 1024)
            {
                return None;
            }
            let mut response = response;
            let mut bytes = Vec::new();
            while let Some(chunk) = response.chunk().await.ok()? {
                if bytes.len() + chunk.len() > 4 * 1024 * 1024 {
                    return None;
                }
                bytes.extend_from_slice(&chunk);
            }
            let image = ImageAsset::from_bytes(&bytes);
            events.send(Event::Cover(url, image)).await.ok()?;
            let callback = ready.lock().unwrap().clone();
            callback();
            Some(())
        });
        // Completion callback is emitted below or when the bounded set yields.
        let callback = wake.lock().unwrap().clone();
        callback();
    }
    while covers.join_next().await.is_some() {
        let callback = wake.lock().unwrap().clone();
        callback();
    }
}
