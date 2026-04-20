# Snapix — Linux Screenshot Beautify App

> Screenshot tool cho Linux vừa annotate nhanh vừa beautify đẹp, native GTK4, Wayland-first.

**Thông tin dự án:**
- **Tên:** Snapix
- **Platform:** Linux (X11 + Wayland)
- **Stack:** Rust + GTK4 + libadwaita
- **License:** Apache-2.0
- **Business model:** Freemium (free base + paid premium)
- **Distribution:** Flatpak (primary), AppImage, AUR

---

## 1. Product Vision

**One-liner:** Flameshot gặp Xnapper — screenshot tool cho Linux vừa annotate nhanh vừa beautify đẹp, native GTK4, Wayland-first.

**Target users:**
- Developers chia sẻ code/UI screenshot lên Twitter/blog
- Content creators viết tutorial
- Designers làm mockup nhanh

**Core differentiators (ưu tiên theo thứ tự):**
1. **Beautify** chất lượng cao — gradient BG, padding, shadow, rounded corners, device mockup
2. **Wayland-native** — hoạt động mượt trên GNOME/KDE/Hyprland
3. **Smart features** — auto-redact secrets (API key/email/credit card), smart crop theo window
4. **Fast workflow** — hotkey → chụp → edit → copy/share, tất cả <3 giây

---

## 2. Feature Scope

### MVP (v0.1)
- [ ] Chụp fullscreen / region / active window
- [ ] Hotkey toàn cục
- [ ] Editor: crop, arrow, rectangle, text, blur region
- [ ] Beautify: gradient/solid BG, padding, corner radius, shadow
- [ ] Export PNG/JPG, copy clipboard
- [ ] Support X11 + Wayland (GNOME, KDE)

### v0.2
- [ ] Auto-redact (OCR qua `tesseract-rs`) — **Pro**
- [ ] Window mockup frames (macOS/Linux window chrome) — **Pro**
- [ ] Upload lên imgur/custom endpoint — **Pro**
- [ ] Hyprland/Sway support

### v1.0
- [ ] Scrolling screenshot — **Pro**
- [ ] Screen recording → GIF/MP4 (qua `gstreamer`) — **Pro**
- [ ] OCR "copy text from image" — **Pro**
- [ ] Template/preset cho BG

### Out of scope (không làm)
- Cloud sync, account system, team features → giữ local-first
- Mobile app
- Photo editor phức tạp kiểu GIMP

---

## 3. Freemium Model

### Phân chia Free vs Pro

| Free | Pro |
|---|---|
| Chụp full/region/window | Scrolling screenshot |
| Annotate cơ bản (arrow, rect, text, blur) | Screen recording → GIF/MP4 |
| Beautify cơ bản (padding, radius, shadow, 5 gradient preset) | AI auto-redact (OCR) |
| Export PNG/JPG, copy clipboard | Unlimited custom templates |
| 3 custom templates | Cloud upload (imgur/S3/custom) |
|  | OCR copy text |
|  | Device/browser mockup frames |
|  | Priority support |

**Nguyên tắc:** Free phải đủ dùng để user không bỏ đi. Pro là "nice to have" cho power user.

### Pricing
- **One-time purchase:** $15-25 (Xnapper $18, CleanShot $29)
- **Lifetime license** dễ bán cho Linux user hơn subscription
- **Free cho student/OSS maintainer** — goodwill marketing

### License key system
- Self-hosted qua **Gumroad** hoặc **Lemon Squeezy**
- App verify offline bằng Ed25519 signing (crate `ed25519-dalek`)
- Không cần server riêng cho MVP

### Open-core strategy
**Option A (recommend cho MVP):** Toàn bộ code Apache-2.0 public, chỉ license key gate Pro features.
- Ai muốn build from source tự bypass được, nhưng ít ai làm
- Cộng đồng OSS ủng hộ → marketing tốt
- Revenue đến từ 95% user không build from source

**Option B (scale về sau):** Core Apache-2.0 public, Pro features trong repo private.

### Flatpak + freemium
- Flathub không cho bán in-app purchase trực tiếp
- Giải pháp: app free trên Flathub, user mua license key trên website → paste vào app để unlock
- Tiền lệ: Apostrophe, Tuba, các app GTK khác

---

## 4. Architecture

```
┌──────────────────────────────────────────────────┐
│  Presentation Layer (GTK4 + libadwaita)          │
│  ├─ Capture overlay (region selector)            │
│  ├─ Editor window (main UI)                      │
│  └─ Preferences dialog                           │
├──────────────────────────────────────────────────┤
│  Application Layer                               │
│  ├─ AppState (Arc<RwLock<...>>)                  │
│  ├─ Command dispatcher (undo/redo stack)         │
│  ├─ Hotkey manager                               │
│  └─ Entitlements / License verifier              │
├──────────────────────────────────────────────────┤
│  Domain / Core                                   │
│  ├─ Canvas model (layers, shapes, transforms)    │
│  ├─ Image pipeline (blur, shadow, gradient)      │
│  └─ Export pipeline                              │
├──────────────────────────────────────────────────┤
│  Platform Abstraction (trait CaptureBackend)     │
│  ├─ X11Backend    (x11rb)                        │
│  ├─ WaylandBackend (ashpd → XDG portal)          │
│  └─ FallbackBackend (spawn grim/gnome-screenshot)│
├──────────────────────────────────────────────────┤
│  System: D-Bus, Clipboard, File I/O              │
└──────────────────────────────────────────────────┘
```

### Module structure
```
snapix/
├── Cargo.toml           (workspace)
├── crates/
│   ├── snapix-core/     # Domain: canvas model, image ops (no GTK)
│   ├── snapix-capture/  # Screenshot backends (X11 + Wayland)
│   ├── snapix-ui/       # GTK4 views & widgets
│   └── snapix-app/      # Binary entry point, wiring
├── data/                # .desktop, icons, GSettings schema
├── po/                  # i18n translations
└── flatpak/             # Flatpak manifest
```

Tách `snapix-core` không phụ thuộc GTK → dễ unit test, sau này có thể port sang Tauri/CLI nếu cần.

---

## 5. Tech Stack & Dependencies

### Core crates
| Crate | Role |
|---|---|
| `gtk4` + `libadwaita` | UI framework, modern GNOME look |
| `glib`, `gio` | Event loop, async |
| `cairo-rs` | 2D rendering cho editor canvas |
| `gdk-pixbuf` | Image loading/saving |
| `image` | Image processing (fallback, format conversion) |
| `ashpd` | XDG Desktop Portal client (Wayland screenshot, global shortcuts) |
| `x11rb` | X11 native protocol |
| `zbus` | D-Bus async client |
| `arboard` | Clipboard cross-desktop |
| `serde` + `serde_json` | Config serialization |
| `async-std` | Async runtime (match với `ashpd` + GTK) |
| `anyhow` + `thiserror` | Error handling |
| `tracing` + `tracing-subscriber` | Logging |
| `ed25519-dalek` | License key verification |

### Image processing
| Crate | Role |
|---|---|
| `tiny-skia` | GPU-like 2D render cho gradient/shadow |
| `fast_image_resize` | SIMD resize |
| `imageproc` | Blur, filter |

### Optional (v0.2+)
| Crate | Role |
|---|---|
| `tesseract` hoặc `leptess` | OCR cho auto-redact |
| `regex` | Detect secrets pattern |
| `gstreamer-rs` | Screen recording |

---

## 6. Key Design Decisions

### 6.1 Capture abstraction
```rust
#[async_trait]
pub trait CaptureBackend: Send + Sync {
    async fn capture_full(&self) -> Result<Image>;
    async fn capture_region(&self, region: Rect) -> Result<Image>;
    async fn capture_window(&self) -> Result<Image>;
    fn supports_interactive(&self) -> bool;
}

pub fn detect_backend() -> Box<dyn CaptureBackend> {
    if is_wayland() {
        Box::new(WaylandBackend::new())  // qua ashpd portal
    } else {
        Box::new(X11Backend::new())      // x11rb trực tiếp
    }
}
```

**Vì sao:** Wayland bắt buộc qua portal → user dialog approve mỗi lần (hoặc "always allow"). X11 tự do hơn. Abstract để UI không cần biết.

### 6.2 Canvas model — document là single source of truth
```rust
pub struct Document {
    pub base_image: Image,              // screenshot gốc
    pub background: Background,         // gradient/solid/image/blur
    pub frame: FrameSettings,           // padding, radius, shadow
    pub annotations: Vec<Annotation>,   // arrow, text, blur box
    pub history: UndoStack<DocumentPatch>,
}

pub enum Annotation {
    Arrow { from: Point, to: Point, color: Color, width: f32 },
    Rect { bounds: Rect, stroke: Stroke, fill: Option<Color> },
    Text { pos: Point, content: String, style: TextStyle },
    Blur { bounds: Rect, radius: f32 },
    Redact { bounds: Rect },
}
```

**Render pipeline:** mỗi frame → render BG → composite base_image với frame → vẽ annotations lên trên. Dùng `tiny-skia` render ra buffer, paint lên GTK `DrawingArea` qua cairo surface.

### 6.3 Undo/redo
Dùng **command pattern** — mỗi action là `DocumentPatch` có thể apply/revert. Không clone cả document.

### 6.4 Global hotkey
- **X11:** `x11rb` + `XGrabKey` hoặc external tool (sxhkd)
- **Wayland:** XDG portal `org.freedesktop.portal.GlobalShortcuts` (GNOME 45+, KDE Plasma 6)
- **Fallback:** hướng dẫn user bind hotkey trong DE settings → gọi `snapix --capture region`

### 6.5 Async model
GTK4 main loop + `glib::MainContext::spawn_local()` cho async task. `ashpd` support cả `tokio` lẫn `async-std` — chọn `async-std` vì integrate GTK tốt hơn qua `gio`.

### 6.6 Entitlement system (freemium)
Thiết kế từ M0 để tránh refactor sau:

```rust
pub enum Tier { Free, Pro }

pub struct Entitlements {
    tier: Tier,
    features: HashSet<Feature>,
}

pub enum Feature {
    UnlimitedExports,
    AiRedact,
    CloudUpload,
    ScreenRecording,
    CustomTemplates,
    ScrollingCapture,
    WindowMockup,
}

// Dùng ở UI:
if entitlements.has(Feature::AiRedact) {
    show_button()
} else {
    show_upgrade_cta()
}
```

---

## 7. UX Flow

### Flow chính
```
[Hotkey pressed]
   ↓
[Overlay fullscreen — darken, crosshair]
   ↓ (user drags region)
[Capture → load vào Editor window]
   ↓
[Editor: toolbar top, canvas center, inspector right]
   ↓
[Export: Ctrl+C copy, Ctrl+S save, Ctrl+U upload]
```

### Editor layout (libadwaita)
```
┌─────────────────────────────────────────────┐
│ AdwHeaderBar [Copy] [Save] [Share] [⋮]      │
├──────────┬─────────────────────┬────────────┤
│          │                     │ Inspector  │
│ Tools    │   Canvas            │ ─────────  │
│ ──────   │   (DrawingArea)     │ Background │
│ Select   │                     │ [gradient] │
│ Crop     │   [ screenshot ]    │            │
│ Arrow    │                     │ Padding    │
│ Rect     │                     │ [slider]   │
│ Text     │                     │            │
│ Blur     │                     │ Shadow     │
│ Redact   │                     │ [toggle]   │
└──────────┴─────────────────────┴────────────┘
```

Dùng `AdwOverlaySplitView` + `AdwToolbarView` cho layout adaptive.

---

## 8. Roadmap

### M0 — Foundation
- Setup workspace, CI (GitHub Actions: build + clippy + test)
- GTK4 hello world + libadwaita
- Capture trait + X11 backend (chụp fullscreen → lưu PNG)
- **Entitlements struct + Feature flag system (stub)**
- Stub `LicenseVerifier` trait
- Decision: async runtime, logging setup

**Ship criteria:** `snapix --capture full -o test.png` hoạt động trên X11.

### M1 — Wayland support
- `ashpd` integration → Screenshot portal
- Runtime detect X11/Wayland
- Flatpak manifest đầu tiên
- Test trên GNOME + KDE Wayland

**Ship criteria:** Chụp được trên cả 2 env, release Flatpak dev build.

### M2 — Editor MVP
- GTK4 editor window với `DrawingArea`
- Canvas model + render pipeline
- 3 tools: crop, arrow, text
- Export PNG + copy clipboard

**Ship criteria:** User flow complete: hotkey → chụp → edit → copy.

**Current implementation snapshot (2026-04-21):**
- Editor shell đã chạy được làm cửa sổ GUI mặc định của app, và startup capture đã được nối vào editor.
- Trên Wayland portal, nếu full-screen capture fail thì app fallback sang interactive window capture để vẫn nạp ảnh vào editor.
- Layout editor đã được làm lại theo hướng dễ dùng hơn: top action row, top tool row, canvas trung tâm, settings panel bên phải.
- `DrawingArea` canvas đã render background, frame padding, corner radius, shadow, crop overlay, arrow, text, và image surface nếu `Document.base_image` có dữ liệu.
- Inspector controls cho padding/radius/shadow/background đang update `Document` realtime.
- `Save` đã export PNG và `Copy` đã copy ảnh render hiện tại vào clipboard.
- Crop đã usable với default crop box, move/resize handles, `Enter` để apply, `Esc` để cancel; tuy nhiên UX/polish vẫn chưa thật sự tốt.
- Arrow đã usable theo flow drag-to-place.
- Text đã usable theo flow click-to-place + dialog nhập nội dung.
- Undo/redo đã hoạt động bằng snapshot toàn `Document`.
- Top action row đã nối được `Fullscreen / Region / Import / Clear`, nhưng capture semantics trên Wayland portal vẫn chưa ổn định: `Fullscreen` có thể fail, `Region` là interactive path chính, và `Window` hiện không nên coi là fully supported trên portal hiện tại.
- Vì vậy ship criteria của M2 vẫn chưa đạt hoàn toàn; phần còn thiếu chính là capture UX ổn định hơn và annotation editing/polish.

### M3 — Beautify
- Background: gradient picker, solid, image, blur-of-screenshot
- Frame: padding slider, corner radius, drop shadow
- Preset system (save/load custom presets)

**Ship criteria:** Output screenshot nhìn "đẹp hơn Xnapper" — demo được trên Twitter.

### M4 — Polish & release v0.1
- Preferences dialog
- i18n (English + Vietnamese)
- Icon, .desktop file, AppStream metadata
- **Website landing page + Gumroad/Lemon Squeezy setup**
- **License key verify flow**
- **"Unlock Pro" dialog trong app**
- Flathub submission

**Ship criteria:** v0.1.0 trên Flathub + website bán license.

### M5 — First paid feature (v0.2)
- **Auto-redact (OCR + regex)** — good first Pro feature, value cao, độc lập
- Window mockup frames
- Upload integration
- More tools (numbered step, spotlight)

### M6+ — Scale
- Scrolling screenshot
- Screen recording
- OCR copy text
- Template marketplace?

---

## 9. Distribution Strategy

### Primary: Flatpak (Flathub)
- Sandbox → Wayland portal work out of the box
- Auto-update
- Manifest ví dụ:
```yaml
app-id: io.github.yourname.Snapix
runtime: org.gnome.Platform
runtime-version: '46'
sdk: org.gnome.Sdk
sdk-extensions:
  - org.freedesktop.Sdk.Extension.rust-stable
command: snapix
finish-args:
  - --socket=wayland
  - --socket=fallback-x11
  - --share=ipc
  - --talk-name=org.freedesktop.portal.Desktop
  - --filesystem=xdg-pictures/Screenshots:create
```

### Secondary
- **AppImage** — portable, non-Flatpak users
- **AUR** — Arch users (package `snapix-git` + `snapix-bin`)
- **.deb / .rpm** qua `cargo-deb` / `cargo-generate-rpm`

### Không ưu tiên
- Snap (phức tạp, ít user overlap)
- COPR / OBS (maintain nhiều → sau)

---

## 10. Testing Strategy

| Layer | Cách test |
|---|---|
| `snapix-core` | Unit test pure functions (canvas math, image ops) |
| `snapix-capture` | Integration test với mock backend + manual test matrix |
| `snapix-ui` | GTK test framework + snapshot test cho render |
| End-to-end | Manual QA matrix |

**QA matrix bắt buộc test mỗi release:**

| DE | Session | Ưu tiên |
|---|---|---|
| GNOME 46+ | Wayland | ⭐⭐⭐ |
| KDE Plasma 6 | Wayland | ⭐⭐⭐ |
| GNOME | X11 | ⭐⭐ |
| Hyprland | Wayland | ⭐⭐ |
| Sway | Wayland | ⭐ |
| XFCE | X11 | ⭐ |

---

## 11. Rủi ro & Mitigation

| Rủi ro | Impact | Mitigation |
|---|---|---|
| Wayland portal UX phiền (dialog mỗi lần chụp) | Cao | Document rõ cho user, push upstream cho `persist` option |
| Global hotkey Wayland chưa chuẩn | Cao | Fallback: user bind DE hotkey gọi CLI |
| GTK4 learning curve | Trung | Có thể bắt đầu với Relm4 (framework Elm-like trên GTK4) |
| Cạnh tranh Flameshot | Trung | Khác biệt rõ ở beautify, không cạnh tranh ở annotate |
| Maintain Linux packaging mệt | Trung | Chỉ commit Flatpak + AppImage. Community maintain AUR/deb |
| Rust build time chậm | Thấp | `sccache`, `mold` linker, tách crate |
| License key bị crack | Thấp | Accept it — focus vào giá trị Pro feature, không DRM cực đoan |
| Flathub reject app bán license | Trung | Tham khảo app GTK đã làm: Apostrophe, Tuba — model free app + external license |

---

## 12. Next concrete steps (week 1)

1. `cargo new --bin snapix && cd snapix` → setup workspace với 4 crate ở mục 4
2. Add deps `gtk4`, `libadwaita`, `x11rb`, `ashpd` — viết hello window
3. Implement `CaptureBackend` trait + `X11Backend::capture_full()`
4. CLI skeleton: `snapix --capture full -o out.png` → verify end-to-end
5. Setup GitHub repo, CI (build + clippy), README, LICENSE (Apache-2.0)
6. Stub `Entitlements` + `LicenseVerifier` từ M0

---

## 13. Competitive Landscape

| Tool | Điểm mạnh | Điểm yếu | Snapix khác biệt |
|---|---|---|---|
| Flameshot | Annotate tốt, free, cross-platform | Không có beautify, UI cũ | Beautify + UI hiện đại |
| Spectacle (KDE) | Tích hợp KDE | Basic | Beautify + GTK native |
| GNOME Screenshot | Đơn giản | Quá basic | Full editor |
| Shutter | Nhiều tính năng | Perl, bảo trì kém | Modern Rust stack |
| Xnapper (macOS) | Beautify chuẩn | Không có Linux | Fill gap này trên Linux |
| CleanShot X (macOS) | All-in-one | Không có Linux | Same |

**Kết luận:** Không có tool Linux nào làm tốt phần beautify. Snapix nhắm đúng gap này.

---

## Appendix A — References

- [Xnapper](https://xnapper.com/) — reference UX
- [Flameshot](https://flameshot.org/) — reference feature set (annotate)
- [ashpd crate](https://docs.rs/ashpd/) — XDG Portal Rust
- [gtk4-rs book](https://gtk-rs.org/gtk4-rs/stable/latest/book/)
- [libadwaita docs](https://gnome.pages.gitlab.gnome.org/libadwaita/)
- [Flathub submission guide](https://docs.flathub.org/docs/for-app-authors/submission)
- [Relm4](https://relm4.org/) — Elm-like wrapper trên GTK4 (tùy chọn)
