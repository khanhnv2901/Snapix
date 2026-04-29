use gtk4::cairo;

#[derive(Clone, Copy)]
pub(crate) enum MeshPalette {
    Vibrant,
    Sunset,
    Candy,
    Aurora,
    Peach,
    Lagoon,
}

#[derive(Clone, Copy)]
struct MeshBlob {
    x: f64,
    y: f64,
    radius: f64,
    color: (f64, f64, f64),
    alpha: f64,
}

struct MeshPaletteSpec {
    base: [(f64, f64, f64); 4],
    blobs: [MeshBlob; 6],
    lift: [(f64, f64, f64, f64); 4],
    wash: (f64, f64, f64),
}

pub(crate) fn paint_vibrant_mesh_background(
    cr: &cairo::Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    palette: MeshPalette,
    intensity: f64,
) {
    let intensity = intensity.clamp(0.2, 1.0);
    let max_dim = width.max(height);
    let spec = mesh_palette_spec(palette);

    let base = cairo::LinearGradient::new(x, y, x + width, y + height);
    for (offset, color) in [0.0, 0.30, 0.68, 1.0].into_iter().zip(spec.base) {
        base.add_color_stop_rgb(offset, color.0, color.1, color.2);
    }
    cr.set_source(&base).ok();
    cr.rectangle(x, y, width, height);
    cr.fill().ok();

    for blob in spec.blobs {
        paint_blob(
            cr,
            x + width * blob.x,
            y + height * blob.y,
            max_dim * blob.radius,
            blob.color,
            blob.alpha * intensity,
        );
    }

    let warm_lift = cairo::LinearGradient::new(x, y + height * 0.18, x + width, y + height * 0.88);
    for (offset, color) in [0.0, 0.36, 0.70, 1.0].into_iter().zip(spec.lift) {
        warm_lift.add_color_stop_rgba(offset, color.0, color.1, color.2, color.3 * intensity);
    }
    cr.set_source(&warm_lift).ok();
    cr.rectangle(x, y, width, height);
    cr.fill().ok();

    cr.set_source_rgba(
        spec.wash.0,
        spec.wash.1,
        spec.wash.2,
        0.04 + 0.05 * intensity,
    );
    cr.rectangle(x + width * 0.62, y, width * 0.38, height);
    cr.fill().ok();
}

fn mesh_palette_spec(palette: MeshPalette) -> MeshPaletteSpec {
    match palette {
        MeshPalette::Vibrant => MeshPaletteSpec {
            base: [
                (1.0, 0.50, 0.00),
                (0.98, 0.02, 0.28),
                (0.99, 0.62, 0.42),
                (0.89, 0.84, 0.96),
            ],
            blobs: [
                blob(0.48, 0.10, 0.52, (1.0, 0.00, 0.30), 0.66),
                blob(0.72, 0.12, 0.44, (1.0, 0.36, 0.18), 0.36),
                blob(0.10, 0.20, 0.48, (1.0, 0.33, 0.00), 0.50),
                blob(0.24, 0.82, 0.48, (1.0, 0.18, 0.36), 0.40),
                blob(0.55, 1.04, 0.44, (1.0, 0.78, 0.20), 0.44),
                blob(1.02, 0.42, 0.52, (0.78, 0.74, 0.98), 0.50),
            ],
            lift: [
                (1.0, 0.12, 0.18, 0.12),
                (1.0, 0.55, 0.16, 0.08),
                (1.0, 0.94, 0.66, 0.12),
                (0.88, 0.84, 1.0, 0.18),
            ],
            wash: (1.0, 1.0, 1.0),
        },
        MeshPalette::Sunset => MeshPaletteSpec {
            base: [
                (1.0, 0.76, 0.36),
                (0.98, 0.30, 0.20),
                (0.72, 0.18, 0.48),
                (0.19, 0.14, 0.38),
            ],
            blobs: [
                blob(0.18, 0.12, 0.50, (1.0, 0.86, 0.36), 0.62),
                blob(0.68, 0.20, 0.48, (1.0, 0.24, 0.18), 0.54),
                blob(0.92, 0.55, 0.52, (0.48, 0.20, 0.74), 0.46),
                blob(0.34, 0.86, 0.54, (1.0, 0.42, 0.18), 0.46),
                blob(0.00, 0.62, 0.46, (0.98, 0.18, 0.38), 0.40),
                blob(0.56, 1.05, 0.48, (1.0, 0.68, 0.32), 0.34),
            ],
            lift: [
                (1.0, 0.78, 0.36, 0.10),
                (1.0, 0.24, 0.20, 0.10),
                (0.94, 0.26, 0.50, 0.12),
                (0.30, 0.18, 0.52, 0.16),
            ],
            wash: (1.0, 0.92, 0.82),
        },
        MeshPalette::Candy => MeshPaletteSpec {
            base: [
                (1.0, 0.66, 0.86),
                (0.95, 0.36, 0.82),
                (0.62, 0.72, 1.0),
                (0.70, 1.0, 0.94),
            ],
            blobs: [
                blob(0.18, 0.18, 0.50, (1.0, 0.38, 0.72), 0.54),
                blob(0.62, 0.08, 0.44, (0.58, 0.50, 1.0), 0.48),
                blob(0.98, 0.38, 0.50, (0.36, 0.95, 0.92), 0.42),
                blob(0.32, 0.84, 0.50, (1.0, 0.78, 0.22), 0.34),
                blob(0.02, 0.62, 0.44, (0.96, 0.24, 0.72), 0.36),
                blob(0.70, 0.92, 0.48, (0.70, 0.94, 1.0), 0.44),
            ],
            lift: [
                (1.0, 0.42, 0.76, 0.10),
                (0.72, 0.48, 1.0, 0.10),
                (0.54, 0.98, 0.92, 0.10),
                (1.0, 0.94, 1.0, 0.16),
            ],
            wash: (1.0, 1.0, 1.0),
        },
        MeshPalette::Aurora => MeshPaletteSpec {
            base: [
                (0.06, 0.16, 0.30),
                (0.05, 0.48, 0.52),
                (0.28, 0.26, 0.70),
                (0.10, 0.12, 0.28),
            ],
            blobs: [
                blob(0.18, 0.22, 0.52, (0.18, 0.95, 0.72), 0.48),
                blob(0.56, 0.10, 0.48, (0.34, 0.54, 1.0), 0.44),
                blob(0.92, 0.32, 0.50, (0.78, 0.38, 1.0), 0.38),
                blob(0.28, 0.82, 0.48, (0.12, 0.78, 0.68), 0.42),
                blob(0.72, 1.02, 0.52, (0.18, 0.24, 0.78), 0.42),
                blob(0.02, 0.62, 0.44, (0.38, 1.0, 0.56), 0.32),
            ],
            lift: [
                (0.20, 0.98, 0.74, 0.10),
                (0.28, 0.48, 1.0, 0.10),
                (0.74, 0.34, 1.0, 0.12),
                (0.08, 0.14, 0.30, 0.18),
            ],
            wash: (0.76, 1.0, 0.94),
        },
        MeshPalette::Peach => MeshPaletteSpec {
            base: [
                (1.0, 0.84, 0.68),
                (1.0, 0.64, 0.58),
                (0.96, 0.78, 0.58),
                (0.82, 0.90, 1.0),
            ],
            blobs: [
                blob(0.18, 0.18, 0.50, (1.0, 0.60, 0.48), 0.48),
                blob(0.62, 0.08, 0.46, (1.0, 0.78, 0.42), 0.36),
                blob(0.92, 0.44, 0.52, (0.68, 0.82, 1.0), 0.42),
                blob(0.32, 0.86, 0.50, (1.0, 0.48, 0.42), 0.36),
                blob(0.04, 0.62, 0.42, (1.0, 0.78, 0.62), 0.40),
                blob(0.70, 1.02, 0.48, (0.92, 0.66, 0.90), 0.34),
            ],
            lift: [
                (1.0, 0.58, 0.46, 0.08),
                (1.0, 0.74, 0.42, 0.08),
                (1.0, 0.92, 0.72, 0.10),
                (0.78, 0.88, 1.0, 0.16),
            ],
            wash: (1.0, 0.96, 0.90),
        },
        MeshPalette::Lagoon => MeshPaletteSpec {
            base: [
                (0.10, 0.72, 0.78),
                (0.10, 0.50, 0.76),
                (0.18, 0.84, 0.62),
                (0.72, 0.94, 0.88),
            ],
            blobs: [
                blob(0.14, 0.18, 0.50, (0.12, 0.86, 0.78), 0.52),
                blob(0.62, 0.10, 0.48, (0.12, 0.44, 0.92), 0.44),
                blob(0.98, 0.44, 0.52, (0.18, 0.94, 0.64), 0.46),
                blob(0.28, 0.84, 0.50, (0.10, 0.64, 0.78), 0.40),
                blob(0.00, 0.60, 0.44, (0.72, 1.0, 0.84), 0.34),
                blob(0.68, 1.04, 0.48, (0.50, 0.86, 1.0), 0.40),
            ],
            lift: [
                (0.16, 0.92, 0.82, 0.10),
                (0.12, 0.46, 0.94, 0.08),
                (0.50, 1.0, 0.72, 0.12),
                (0.88, 1.0, 0.96, 0.16),
            ],
            wash: (0.90, 1.0, 0.96),
        },
    }
}

fn blob(x: f64, y: f64, radius: f64, color: (f64, f64, f64), alpha: f64) -> MeshBlob {
    MeshBlob {
        x,
        y,
        radius,
        color,
        alpha,
    }
}

fn paint_blob(
    cr: &cairo::Context,
    center_x: f64,
    center_y: f64,
    radius: f64,
    color: (f64, f64, f64),
    alpha: f64,
) {
    let blob = cairo::RadialGradient::new(
        center_x,
        center_y,
        radius * 0.04,
        center_x,
        center_y,
        radius,
    );
    blob.add_color_stop_rgba(0.0, color.0, color.1, color.2, alpha);
    blob.add_color_stop_rgba(0.34, color.0, color.1, color.2, alpha * 0.62);
    blob.add_color_stop_rgba(0.74, color.0, color.1, color.2, alpha * 0.16);
    blob.add_color_stop_rgba(1.0, color.0, color.1, color.2, 0.0);
    cr.set_source(&blob).ok();
    cr.arc(center_x, center_y, radius, 0.0, std::f64::consts::TAU);
    cr.fill().ok();
}
