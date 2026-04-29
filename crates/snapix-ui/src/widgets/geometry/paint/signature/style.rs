use snapix_core::canvas::BackgroundStyleId;

#[derive(Clone, Copy)]
pub(super) struct SignatureShadowProfile {
    pub(super) blur_scale: f64,
    pub(super) strength_scale: f64,
}

#[derive(Clone, Copy)]
pub(super) struct SignaturePreviewPalette {
    pub(super) fill_rgba: (f64, f64, f64, f64),
    pub(super) stroke_rgba: (f64, f64, f64, f64),
}

pub(super) fn shadow_profile(id: BackgroundStyleId, intensity: f64) -> SignatureShadowProfile {
    let intensity = intensity.clamp(0.2, 1.0);
    match id {
        BackgroundStyleId::Blueprint => SignatureShadowProfile {
            blur_scale: 1.0 + intensity * 0.08,
            strength_scale: 1.0 + intensity * 0.16,
        },
        BackgroundStyleId::MidnightPanel => SignatureShadowProfile {
            blur_scale: 1.02 + intensity * 0.20,
            strength_scale: 1.0 + intensity * 0.22,
        },
        BackgroundStyleId::CutPaper => SignatureShadowProfile {
            blur_scale: 0.92 - intensity * 0.10,
            strength_scale: 0.98 - intensity * 0.10,
        },
        BackgroundStyleId::TerminalGlow => SignatureShadowProfile {
            blur_scale: 1.04 + intensity * 0.28,
            strength_scale: 1.0 + intensity * 0.12,
        },
        BackgroundStyleId::Redacted => SignatureShadowProfile {
            blur_scale: 0.92 + intensity * 0.08,
            strength_scale: 1.0 + intensity * 0.08,
        },
    }
}

pub(super) fn preview_palette(id: BackgroundStyleId) -> SignaturePreviewPalette {
    match id {
        BackgroundStyleId::CutPaper => SignaturePreviewPalette {
            fill_rgba: (0.99, 0.98, 0.95, 0.92),
            stroke_rgba: (0.20, 0.18, 0.16, 0.16),
        },
        BackgroundStyleId::TerminalGlow => SignaturePreviewPalette {
            fill_rgba: (0.08, 0.14, 0.14, 0.90),
            stroke_rgba: (0.28, 0.96, 0.76, 0.16),
        },
        BackgroundStyleId::Redacted => SignaturePreviewPalette {
            fill_rgba: (0.12, 0.13, 0.16, 0.90),
            stroke_rgba: (0.92, 0.94, 0.98, 0.10),
        },
        BackgroundStyleId::Blueprint | BackgroundStyleId::MidnightPanel => {
            SignaturePreviewPalette {
                fill_rgba: (0.96, 0.97, 1.0, 0.90),
                stroke_rgba: (0.82, 0.88, 1.0, 0.12),
            }
        }
    }
}
