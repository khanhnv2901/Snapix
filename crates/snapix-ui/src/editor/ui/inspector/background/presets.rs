use gtk4::prelude::*;
use snapix_core::canvas::{Background, BackgroundStyleId, Color};

use crate::widgets::paint_signature_preview_thumbnail;

pub(super) struct BackgroundPresetDefinition {
    pub(super) label: &'static str,
    pub(super) css_class: &'static str,
    pub(super) background: Background,
}

pub(super) fn background_presets() -> Vec<BackgroundPresetDefinition> {
    vec![
        BackgroundPresetDefinition {
            label: "Cornflower",
            css_class: "swatch-cornflower",
            background: Background::Gradient {
                from: Color {
                    r: 110,
                    g: 162,
                    b: 255,
                    a: 255,
                },
                to: Color {
                    r: 130,
                    g: 99,
                    b: 245,
                    a: 255,
                },
                angle_deg: 135.0,
            },
        },
        BackgroundPresetDefinition {
            label: "Sunset",
            css_class: "swatch-sunset",
            background: Background::Gradient {
                from: Color {
                    r: 255,
                    g: 180,
                    b: 108,
                    a: 255,
                },
                to: Color {
                    r: 232,
                    g: 93,
                    b: 68,
                    a: 255,
                },
                angle_deg: 135.0,
            },
        },
        BackgroundPresetDefinition {
            label: "Ocean",
            css_class: "swatch-ocean",
            background: Background::Gradient {
                from: Color {
                    r: 56,
                    g: 189,
                    b: 248,
                    a: 255,
                },
                to: Color {
                    r: 15,
                    g: 118,
                    b: 110,
                    a: 255,
                },
                angle_deg: 135.0,
            },
        },
        BackgroundPresetDefinition {
            label: "Forest",
            css_class: "swatch-forest",
            background: Background::Gradient {
                from: Color {
                    r: 74,
                    g: 222,
                    b: 128,
                    a: 255,
                },
                to: Color {
                    r: 21,
                    g: 128,
                    b: 61,
                    a: 255,
                },
                angle_deg: 135.0,
            },
        },
        BackgroundPresetDefinition {
            label: "Rose",
            css_class: "swatch-rose",
            background: Background::Gradient {
                from: Color {
                    r: 249,
                    g: 168,
                    b: 212,
                    a: 255,
                },
                to: Color {
                    r: 190,
                    g: 24,
                    b: 93,
                    a: 255,
                },
                angle_deg: 135.0,
            },
        },
        BackgroundPresetDefinition {
            label: "Midnight",
            css_class: "swatch-midnight",
            background: Background::Gradient {
                from: Color {
                    r: 99,
                    g: 102,
                    b: 241,
                    a: 255,
                },
                to: Color {
                    r: 30,
                    g: 27,
                    b: 75,
                    a: 255,
                },
                angle_deg: 135.0,
            },
        },
        BackgroundPresetDefinition {
            label: "Golden",
            css_class: "swatch-golden",
            background: Background::Gradient {
                from: Color {
                    r: 251,
                    g: 191,
                    b: 36,
                    a: 255,
                },
                to: Color {
                    r: 180,
                    g: 83,
                    b: 9,
                    a: 255,
                },
                angle_deg: 135.0,
            },
        },
        BackgroundPresetDefinition {
            label: "Lavender",
            css_class: "swatch-lavender",
            background: Background::Gradient {
                from: Color {
                    r: 196,
                    g: 181,
                    b: 253,
                    a: 255,
                },
                to: Color {
                    r: 124,
                    g: 58,
                    b: 237,
                    a: 255,
                },
                angle_deg: 135.0,
            },
        },
        BackgroundPresetDefinition {
            label: "Mint",
            css_class: "swatch-mint",
            background: Background::Gradient {
                from: Color {
                    r: 110,
                    g: 231,
                    b: 183,
                    a: 255,
                },
                to: Color {
                    r: 13,
                    g: 148,
                    b: 136,
                    a: 255,
                },
                angle_deg: 135.0,
            },
        },
        BackgroundPresetDefinition {
            label: "Slate",
            css_class: "swatch-slate",
            background: Background::Solid {
                color: Color {
                    r: 31,
                    g: 36,
                    b: 45,
                    a: 255,
                },
            },
        },
        BackgroundPresetDefinition {
            label: "Charcoal",
            css_class: "swatch-charcoal",
            background: Background::Solid {
                color: Color {
                    r: 45,
                    g: 55,
                    b: 72,
                    a: 255,
                },
            },
        },
        BackgroundPresetDefinition {
            label: "Steel",
            css_class: "swatch-steel",
            background: Background::Solid {
                color: Color {
                    r: 71,
                    g: 85,
                    b: 105,
                    a: 255,
                },
            },
        },
        BackgroundPresetDefinition {
            label: "Mist",
            css_class: "swatch-mist",
            background: Background::Solid {
                color: Color {
                    r: 226,
                    g: 232,
                    b: 240,
                    a: 255,
                },
            },
        },
        BackgroundPresetDefinition {
            label: "Sky",
            css_class: "swatch-sky",
            background: Background::Solid {
                color: Color {
                    r: 56,
                    g: 189,
                    b: 248,
                    a: 255,
                },
            },
        },
        BackgroundPresetDefinition {
            label: "Emerald",
            css_class: "swatch-emerald",
            background: Background::Solid {
                color: Color {
                    r: 16,
                    g: 185,
                    b: 129,
                    a: 255,
                },
            },
        },
        BackgroundPresetDefinition {
            label: "Coral",
            css_class: "swatch-coral",
            background: Background::Solid {
                color: Color {
                    r: 251,
                    g: 113,
                    b: 133,
                    a: 255,
                },
            },
        },
        BackgroundPresetDefinition {
            label: "Amber",
            css_class: "swatch-amber",
            background: Background::Solid {
                color: Color {
                    r: 245,
                    g: 158,
                    b: 11,
                    a: 255,
                },
            },
        },
        BackgroundPresetDefinition {
            label: "Violet",
            css_class: "swatch-violet",
            background: Background::Solid {
                color: Color {
                    r: 139,
                    g: 92,
                    b: 246,
                    a: 255,
                },
            },
        },
        BackgroundPresetDefinition {
            label: "Deep Space",
            css_class: "swatch-deepspace",
            background: Background::Gradient {
                from: Color {
                    r: 26,
                    g: 26,
                    b: 46,
                    a: 255,
                },
                to: Color {
                    r: 22,
                    g: 33,
                    b: 62,
                    a: 255,
                },
                angle_deg: 135.0,
            },
        },
        BackgroundPresetDefinition {
            label: "Aurora",
            css_class: "swatch-aurora",
            background: Background::Gradient {
                from: Color {
                    r: 34,
                    g: 211,
                    b: 238,
                    a: 255,
                },
                to: Color {
                    r: 16,
                    g: 185,
                    b: 129,
                    a: 255,
                },
                angle_deg: 135.0,
            },
        },
        BackgroundPresetDefinition {
            label: "Blueprint",
            css_class: "swatch-blueprint",
            background: Background::Style {
                id: BackgroundStyleId::Blueprint,
                intensity: 0.65,
            },
        },
        BackgroundPresetDefinition {
            label: "Midnight Panel",
            css_class: "swatch-midnightpanel",
            background: Background::Style {
                id: BackgroundStyleId::MidnightPanel,
                intensity: 0.65,
            },
        },
        BackgroundPresetDefinition {
            label: "Cut Paper",
            css_class: "swatch-cutpaper",
            background: Background::Style {
                id: BackgroundStyleId::CutPaper,
                intensity: 0.65,
            },
        },
        BackgroundPresetDefinition {
            label: "Terminal Glow",
            css_class: "swatch-terminalglow",
            background: Background::Style {
                id: BackgroundStyleId::TerminalGlow,
                intensity: 0.65,
            },
        },
        BackgroundPresetDefinition {
            label: "Redacted",
            css_class: "swatch-redacted",
            background: Background::Style {
                id: BackgroundStyleId::Redacted,
                intensity: 0.65,
            },
        },
        BackgroundPresetDefinition {
            label: "Warning Tape",
            css_class: "swatch-warningtape",
            background: Background::Style {
                id: BackgroundStyleId::WarningTape,
                intensity: 0.65,
            },
        },
    ]
}

pub(super) fn build_signature_preview_card(label: &str, background: &Background) -> gtk4::Widget {
    let card = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(8)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(8)
        .margin_end(8)
        .build();

    let art = gtk4::DrawingArea::new();
    art.set_height_request(68);
    let preview_background = background.clone();
    art.set_draw_func(move |_, cr, width, height| {
        paint_signature_preview_thumbnail(cr, width as f64, height as f64, &preview_background);
    });

    let title = gtk4::Label::builder()
        .label(label)
        .xalign(0.0)
        .wrap(true)
        .css_classes(["background-swatch-label"])
        .build();
    if matches!(
        background,
        Background::Style {
            id: BackgroundStyleId::CutPaper,
            ..
        }
    ) {
        title.add_css_class("background-swatch-label-dark");
    }

    card.append(&art);
    card.append(&title);
    card.upcast()
}
