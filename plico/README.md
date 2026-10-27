# bili-mini · Plico frontend

An experimental alternative to the Qt/QML frontend. The original Qt build and
launcher remain available. This frontend defaults to the **Skia Vulkan** renderer.

## Run

Requires Rust, libmpv (including its linker library), and the usual Plico native
window/font dependencies. Plico dependencies are fetched from Git and pinned to a tested commit in
`Cargo.toml` and `Cargo.lock`. A separate local Plico checkout is not required.
The shared `bili-core` API remains a path dependency within this repository.

```sh
./run-plico.sh
./run-plico.sh --mini
./run-plico.sh --bvid BVxxxxxxxxxx
./run-plico.sh --video /path/to/video.mp4
# Release build:
cargo run --release --manifest-path plico/Cargo.toml
```

The compact window is 480×270 (16:9): video fills the window and playback
controls float over a bottom gradient. Moving the pointer reveals controls;
they fade away after 2.4 seconds of inactivity, and remain visible while paused,
buffering, or hovering the control strip. Expand and close are overlay buttons.
Selecting a video automatically opens compact playback. Drag the picture itself
to move the window; in expanded mode drag the left side of the custom header.
The outer border uses system resizing. Space toggles pause, arrows seek/change
volume, and Escape toggles compact mode.
Shortcuts are fallback actions and do not override an active text editor.

The 登录 button explicitly imports browser cookies using the existing Rust core.
On startup the core reads the same local login cache as the Qt version. No token
or playback URL is intentionally logged. `--video` skips Bilibili API requests.

## Implemented experiment

- Shared `bili-core` library; no duplicated API implementation or HTTP sidecar.
- Recommendations, refresh, video search, cached covers, basic comments.
- Existing login cache and explicit browser-cookie import.
- Separate DASH video/audio URLs plus Bilibili Referer and User-Agent.
- Play/pause, seek, volume, compact window and always-on-top native window.
- Scrolling XML danmaku, original colors, playback-clock interpolation, toggle,
  pause/seek synchronization, bounded tracks and 64 active comments maximum.
- Framework-owned video surface; the app never implements renderer Widget.

API requests run in a cancellable host worker. List/search and stream requests
have generation tickets so late results cannot replace the current selection.
Cover downloads have a four-request concurrency bound, a 4 MiB response limit,
and a 64-entry in-memory image cache. The bounded completion queue provides
backpressure. Nothing is downloaded or decoded in `show`.

This uses Plico's reference **software decoding/output** path at 1280×720 SDR,
followed by Skia GPU upload/composition. It is not hardware-decoded or zero-copy;
performance should not be compared with the Qt/OpenGL frontend until measured.
Vello video playback is not certified by this experiment.

Still missing relative to Qt: UP homepages, fullscreen, auto-next,
tray/global shortcut, double-click expansion, and aspect-locked system resizing.
Platform-specific window positioning is best effort, especially on Wayland.
Danmaku currently supports scrolling modes 1–3; fixed and advanced modes are skipped.
Track spacing uses conservative text-width estimates, not exact glyph measurement.

## Manual verification

```sh
./run-plico.sh --video /path/to/video.mp4 --silent --quit-after 10
./run-plico.sh --bvid BVxxxxxxxxxx --silent --quit-after 20
```

`--opaque` disables transparent corners on surfaces without alpha support
(for example an Xvfb verification run). `--silent` selects mpv's null audio device for headless verification. It still
decodes audio. On shutdown the app reports item/cover counts, video frames,
playback position and decoded audio channel count, without logging stream URLs.
Screenshots/logs belong in the ignored `reports/` directory. Native-window tests
are manual and are not added to CI.

## License

The existing project and shared API implementation retain their MIT license.
The Plico frontend package is distributed under GPL-3.0-or-later (see `LICENSE`),
compatible with using a GPL-enabled libmpv build. Native library binaries and
Rust bindings retain their own licenses and distribution obligations. This
experiment does not bundle libmpv or constitute a completed release package.

## Verification notes

On Linux, the actual API returned 12 recommendation items and 12 covers. A manual
Xvfb/Xfwm run verified header dragging, compact window resizing and Bilibili DASH
playback through Skia; an inspected screenshot showed the video itself. The
null audio output reported two decoded audio channels. This is a functional
check, not a CPU benchmark or an audible synchronization check.

Regression tests cover drag hit regions and valid component identities while
switching between expanded and compact playback. `controls` is a component too:
its nested reusable icon buttons need separate call-site scopes. The latter test
reproduced the duplicate-node error before the fix and passes after it.
