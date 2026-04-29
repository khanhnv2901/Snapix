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
    match id {
        BackgroundStyleId::Blueprint => SignatureShadowProfile {
            blur_scale: 1.1 + 0.15 * intensity,
            strength_scale: 1.15 + 0.20 * intensity,
        },
        BackgroundStyleId::MidnightPanel => SignatureShadowProfile {
            blur_scale: 1.3 + 0.25 * intensity,
            strength_scale: 1.25 + 0.35 * intensity,
        },
        BackgroundStyleId::CutPaper => SignatureShadowProfile {
            blur_scale: 0.65 + 0.10 * intensity,
            strength_scale: 0.75 + 0.15 * intensity,
        },
        BackgroundStyleId::NeoBrutalism => SignatureShadowProfile {
            blur_scale: 0.58 + 0.08 * intensity,
            strength_scale: 1.35 + 0.30 * intensity,
        },
        BackgroundStyleId::MemphisGrid => SignatureShadowProfile {
            blur_scale: 0.82 + 0.10 * intensity,
            strength_scale: 0.95 + 0.18 * intensity,
        },
        BackgroundStyleId::SwissPoster => SignatureShadowProfile {
            blur_scale: 0.72 + 0.08 * intensity,
            strength_scale: 0.82 + 0.10 * intensity,
        },
        BackgroundStyleId::VibrantMesh
        | BackgroundStyleId::SunsetMesh
        | BackgroundStyleId::CandyMesh
        | BackgroundStyleId::AuroraMesh
        | BackgroundStyleId::PeachMesh
        | BackgroundStyleId::LagoonMesh => SignatureShadowProfile {
            blur_scale: 1.18 + 0.16 * intensity,
            strength_scale: 1.05 + 0.14 * intensity,
        },
        BackgroundStyleId::InkWash => SignatureShadowProfile {
            blur_scale: 1.18 + 0.18 * intensity,
            strength_scale: 0.92 + 0.14 * intensity,
        },
        BackgroundStyleId::LiquidGlass => SignatureShadowProfile {
            blur_scale: 1.28 + 0.20 * intensity,
            strength_scale: 0.82 + 0.12 * intensity,
        },
        BackgroundStyleId::TerminalGlow => SignatureShadowProfile {
            blur_scale: 1.2 + 0.20 * intensity,
            strength_scale: 1.0 + 0.25 * intensity,
        },
        BackgroundStyleId::Redacted => SignatureShadowProfile {
            blur_scale: 0.95 + 0.10 * intensity,
            strength_scale: 1.1 + 0.15 * intensity,
        },
        BackgroundStyleId::WarningTape => SignatureShadowProfile {
            blur_scale: 1.1 + 0.10 * intensity,
            strength_scale: 1.4 + 0.30 * intensity,
        },
    }
}

pub(super) fn preview_palette(id: BackgroundStyleId) -> SignaturePreviewPalette {
    match id {
        BackgroundStyleId::CutPaper => SignaturePreviewPalette {
            fill_rgba: (1.0, 1.0, 1.0, 0.95),
            stroke_rgba: (0.12, 0.14, 0.18, 0.15),
        },
        BackgroundStyleId::TerminalGlow => SignaturePreviewPalette {
            fill_rgba: (0.05, 0.10, 0.08, 0.95),
            stroke_rgba: (0.20, 0.95, 0.72, 0.18),
        },
        BackgroundStyleId::NeoBrutalism => SignaturePreviewPalette {
            fill_rgba: (0.99, 0.97, 0.91, 0.95),
            stroke_rgba: (0.08, 0.08, 0.10, 0.28),
        },
        BackgroundStyleId::MemphisGrid => SignaturePreviewPalette {
            fill_rgba: (0.99, 0.98, 0.93, 0.95),
            stroke_rgba: (0.18, 0.20, 0.24, 0.18),
        },
        BackgroundStyleId::SwissPoster => SignaturePreviewPalette {
            fill_rgba: (0.96, 0.96, 0.93, 0.95),
            stroke_rgba: (0.16, 0.18, 0.20, 0.16),
        },
        BackgroundStyleId::VibrantMesh
        | BackgroundStyleId::SunsetMesh
        | BackgroundStyleId::CandyMesh
        | BackgroundStyleId::AuroraMesh
        | BackgroundStyleId::PeachMesh
        | BackgroundStyleId::LagoonMesh => SignaturePreviewPalette {
            fill_rgba: (0.08, 0.09, 0.11, 0.92),
            stroke_rgba: (1.0, 1.0, 1.0, 0.12),
        },
        BackgroundStyleId::InkWash => SignaturePreviewPalette {
            fill_rgba: (1.0, 0.99, 0.96, 0.92),
            stroke_rgba: (0.20, 0.24, 0.30, 0.12),
        },
        BackgroundStyleId::LiquidGlass => SignaturePreviewPalette {
            fill_rgba: (0.98, 1.0, 1.0, 0.78),
            stroke_rgba: (1.0, 1.0, 1.0, 0.30),
        },
        BackgroundStyleId::Redacted => SignaturePreviewPalette {
            fill_rgba: (0.94, 0.96, 1.0, 0.95),
            stroke_rgba: (0.12, 0.15, 0.20, 0.16),
        },
        BackgroundStyleId::WarningTape => SignaturePreviewPalette {
            fill_rgba: (0.12, 0.14, 0.18, 0.96),
            stroke_rgba: (0.98, 0.82, 0.12, 0.25),
        },
        BackgroundStyleId::Blueprint | BackgroundStyleId::MidnightPanel => {
            SignaturePreviewPalette {
                fill_rgba: (0.96, 0.97, 1.0, 0.90),
                stroke_rgba: (0.82, 0.88, 1.0, 0.12),
            }
        }
    }
}
