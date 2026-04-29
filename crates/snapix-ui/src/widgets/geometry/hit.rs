use snapix_core::canvas::Annotation;

use super::super::{CanvasLayout, CropInteractionMode};
use super::layout::{annotation_rect_to_widget_bounds, expand_bounds, point_in_bounds};
use snapix_core::canvas::Document;

pub(crate) fn hit_test_annotation(
    document: &Document,
    layout: CanvasLayout,
    pointer_x: f64,
    pointer_y: f64,
) -> Option<usize> {
    for index in (0..document.annotations.len()).rev() {
        let Some(annotation) = document.annotations.get(index) else {
            continue;
        };
        if matches!(annotation, Annotation::Text { .. })
            && annotation_widget_bounds(document, layout, index).is_some_and(|bounds| {
                point_in_bounds(pointer_x, pointer_y, expand_bounds(bounds, 6.0))
            })
        {
            return Some(index);
        }
    }

    for index in (0..document.annotations.len()).rev() {
        if annotation_widget_bounds(document, layout, index).is_some_and(|bounds| {
            point_in_bounds(pointer_x, pointer_y, expand_bounds(bounds, 10.0))
        }) {
            if let Some(annotation) = document.annotations.get(index) {
                if matches!(
                    annotation,
                    Annotation::Arrow { .. } | Annotation::Line { .. }
                ) {
                    if arrow_hit_test(layout, annotation, pointer_x, pointer_y) {
                        return Some(index);
                    }
                } else {
                    return Some(index);
                }
            }
        }
    }
    None
}

fn annotation_widget_bounds(
    document: &Document,
    layout: CanvasLayout,
    index: usize,
) -> Option<(f64, f64, f64, f64)> {
    let annotation = document.annotations.get(index)?;
    match annotation {
        Annotation::Arrow {
            from, to, width, ..
        }
        | Annotation::Line {
            from, to, width, ..
        } => {
            let start_x = layout.image_x + from.x as f64 * layout.image_scale;
            let start_y = layout.image_y + from.y as f64 * layout.image_scale;
            let end_x = layout.image_x + to.x as f64 * layout.image_scale;
            let end_y = layout.image_y + to.y as f64 * layout.image_scale;
            let padding = (*width as f64).max(12.0);
            let left = start_x.min(end_x) - padding;
            let top = start_y.min(end_y) - padding;
            let width = (start_x - end_x).abs() + padding * 2.0;
            let height = (start_y - end_y).abs() + padding * 2.0;
            Some((left, top, width.max(24.0), height.max(24.0)))
        }
        Annotation::Rect { bounds, stroke, .. } | Annotation::Ellipse { bounds, stroke, .. } => {
            let padding = (stroke.width as f64).max(8.0);
            Some(expand_bounds(
                annotation_rect_to_widget_bounds(layout, bounds),
                padding,
            ))
        }
        Annotation::Blur { bounds, .. } | Annotation::Redact { bounds } => Some(expand_bounds(
            annotation_rect_to_widget_bounds(layout, bounds),
            8.0,
        )),
        Annotation::Text {
            pos,
            content,
            style,
        } => {
            let draw_x = layout.image_x + pos.x as f64 * layout.image_scale;
            let draw_y = layout.image_y + pos.y as f64 * layout.image_scale;
            let font_size = (style.font_size as f64 * layout.image_scale).max(14.0);
            let width = (content.chars().count() as f64 * font_size * 0.62).max(font_size * 1.2);
            let height = font_size * 1.3;
            Some((draw_x - 8.0, draw_y - height, width + 16.0, height + 12.0))
        }
    }
}

pub(crate) fn selection_annotation_widget_bounds(
    document: &Document,
    layout: CanvasLayout,
    index: usize,
) -> Option<(f64, f64, f64, f64)> {
    resizable_annotation_widget_bounds(document, layout, index)
        .or_else(|| annotation_widget_bounds(document, layout, index))
}

pub(crate) fn resizable_annotation_widget_bounds(
    document: &Document,
    layout: CanvasLayout,
    index: usize,
) -> Option<(f64, f64, f64, f64)> {
    let annotation = document.annotations.get(index)?;
    match annotation {
        Annotation::Rect { bounds, .. }
        | Annotation::Ellipse { bounds, .. }
        | Annotation::Blur { bounds, .. }
        | Annotation::Redact { bounds } => Some(annotation_rect_to_widget_bounds(layout, bounds)),
        _ => None,
    }
}

pub(crate) fn hit_resize_handle(
    bounds: (f64, f64, f64, f64),
    pointer_x: f64,
    pointer_y: f64,
) -> Option<CropInteractionMode> {
    let (x, y, width, height) = bounds;
    let right = x + width;
    let bottom = y + height;
    if near_handle(pointer_x, pointer_y, x, y) {
        return Some(CropInteractionMode::ResizeTopLeft);
    }
    if near_handle(pointer_x, pointer_y, right, y) {
        return Some(CropInteractionMode::ResizeTopRight);
    }
    if near_handle(pointer_x, pointer_y, x, bottom) {
        return Some(CropInteractionMode::ResizeBottomLeft);
    }
    if near_handle(pointer_x, pointer_y, right, bottom) {
        return Some(CropInteractionMode::ResizeBottomRight);
    }
    None
}

pub(crate) fn hit_arrow_resize_handle(
    layout: CanvasLayout,
    annotation: &Annotation,
    pointer_x: f64,
    pointer_y: f64,
) -> Option<bool> {
    let (from, to) = match annotation {
        Annotation::Arrow { from, to, .. } | Annotation::Line { from, to, .. } => (from, to),
        _ => return None,
    };
    let start_x = layout.image_x + from.x as f64 * layout.image_scale;
    let start_y = layout.image_y + from.y as f64 * layout.image_scale;
    let end_x = layout.image_x + to.x as f64 * layout.image_scale;
    let end_y = layout.image_y + to.y as f64 * layout.image_scale;
    if near_handle(pointer_x, pointer_y, start_x, start_y) {
        Some(true)
    } else if near_handle(pointer_x, pointer_y, end_x, end_y) {
        Some(false)
    } else {
        None
    }
}

fn arrow_hit_test(
    layout: CanvasLayout,
    annotation: &Annotation,
    pointer_x: f64,
    pointer_y: f64,
) -> bool {
    let (from, to, width) = match annotation {
        Annotation::Arrow {
            from, to, width, ..
        }
        | Annotation::Line {
            from, to, width, ..
        } => (from, to, width),
        _ => return false,
    };
    let start_x = layout.image_x + from.x as f64 * layout.image_scale;
    let start_y = layout.image_y + from.y as f64 * layout.image_scale;
    let end_x = layout.image_x + to.x as f64 * layout.image_scale;
    let end_y = layout.image_y + to.y as f64 * layout.image_scale;
    let tolerance = (*width as f64).max(12.0);
    distance_to_segment(pointer_x, pointer_y, start_x, start_y, end_x, end_y) <= tolerance
}

fn distance_to_segment(px: f64, py: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    if dx == 0.0 && dy == 0.0 {
        return ((px - x1).powi(2) + (py - y1).powi(2)).sqrt();
    }
    let t = (((px - x1) * dx) + ((py - y1) * dy)) / (dx * dx + dy * dy);
    let t = t.clamp(0.0, 1.0);
    let proj_x = x1 + t * dx;
    let proj_y = y1 + t * dy;
    ((px - proj_x).powi(2) + (py - proj_y).powi(2)).sqrt()
}

fn near_handle(pointer_x: f64, pointer_y: f64, handle_x: f64, handle_y: f64) -> bool {
    const HIT_RADIUS: f64 = 18.0;
    (pointer_x - handle_x).abs() <= HIT_RADIUS && (pointer_y - handle_y).abs() <= HIT_RADIUS
}

#[cfg(test)]
mod tests {
    use snapix_core::canvas::{Annotation, Color, Point, Stroke};

    use super::{
        hit_arrow_resize_handle, hit_resize_handle, hit_test_annotation,
        resizable_annotation_widget_bounds,
    };
    use crate::widgets::test_support::{sample_document, sample_layout};
    use crate::widgets::CropInteractionMode;

    #[test]
    fn arrow_resize_handle_detects_start_and_end_points() {
        let layout = sample_layout();
        let annotation = Annotation::Arrow {
            from: Point { x: 5.0, y: 8.0 },
            to: Point { x: 40.0, y: 24.0 },
            color: Color {
                r: 255,
                g: 98,
                b: 54,
                a: 255,
            },
            width: 6.0,
        };

        assert_eq!(
            hit_arrow_resize_handle(layout, &annotation, 20.0, 36.0),
            Some(true)
        );
        assert_eq!(
            hit_arrow_resize_handle(layout, &annotation, 90.0, 68.0),
            Some(false)
        );
        assert_eq!(
            hit_arrow_resize_handle(layout, &annotation, 55.0, 50.0),
            None
        );
    }

    #[test]
    fn line_resize_handle_detects_start_and_end_points() {
        let layout = sample_layout();
        let annotation = Annotation::Line {
            from: Point { x: 5.0, y: 8.0 },
            to: Point { x: 40.0, y: 24.0 },
            color: Color {
                r: 255,
                g: 98,
                b: 54,
                a: 255,
            },
            width: 6.0,
        };

        assert_eq!(
            hit_arrow_resize_handle(layout, &annotation, 20.0, 36.0),
            Some(true)
        );
        assert_eq!(
            hit_arrow_resize_handle(layout, &annotation, 90.0, 68.0),
            Some(false)
        );
    }

    #[test]
    fn resize_handle_detects_expected_corner() {
        let bounds = (30.0, 40.0, 80.0, 60.0);

        assert!(matches!(
            hit_resize_handle(bounds, 30.0, 40.0),
            Some(CropInteractionMode::ResizeTopLeft)
        ));
        assert!(matches!(
            hit_resize_handle(bounds, 110.0, 100.0),
            Some(CropInteractionMode::ResizeBottomRight)
        ));
        assert!(hit_resize_handle(bounds, 70.0, 70.0).is_none());
    }

    #[test]
    fn hit_test_annotation_prefers_topmost_matching_annotation() {
        let mut document = sample_document(100, 80);
        document.annotations.push(Annotation::Rect {
            bounds: snapix_core::canvas::Rect {
                x: 10.0,
                y: 10.0,
                width: 20.0,
                height: 16.0,
            },
            stroke: Stroke {
                color: Color {
                    r: 1,
                    g: 2,
                    b: 3,
                    a: 255,
                },
                width: 4.0,
            },
            fill: None,
        });
        document.annotations.push(Annotation::Ellipse {
            bounds: snapix_core::canvas::Rect {
                x: 12.0,
                y: 12.0,
                width: 18.0,
                height: 14.0,
            },
            stroke: Stroke {
                color: Color {
                    r: 4,
                    g: 5,
                    b: 6,
                    a: 255,
                },
                width: 4.0,
            },
            fill: None,
        });

        assert_eq!(
            hit_test_annotation(&document, sample_layout(), 45.0, 50.0),
            Some(1)
        );
    }

    #[test]
    fn resizable_bounds_exist_for_ellipse_but_not_text() {
        let mut document = sample_document(100, 80);
        document.annotations.push(Annotation::Ellipse {
            bounds: snapix_core::canvas::Rect {
                x: 6.0,
                y: 8.0,
                width: 20.0,
                height: 12.0,
            },
            stroke: Stroke {
                color: Color {
                    r: 255,
                    g: 98,
                    b: 54,
                    a: 255,
                },
                width: 5.0,
            },
            fill: None,
        });
        document.annotations.push(Annotation::Text {
            pos: Point { x: 8.0, y: 10.0 },
            content: "hello".into(),
            style: snapix_core::canvas::TextStyle {
                font_family: "Sans".into(),
                font_size: 24.0,
                color: Color {
                    r: 255,
                    g: 98,
                    b: 54,
                    a: 255,
                },
                bold: true,
            },
        });

        assert!(resizable_annotation_widget_bounds(&document, sample_layout(), 0).is_some());
        assert!(resizable_annotation_widget_bounds(&document, sample_layout(), 1).is_none());
    }
}
