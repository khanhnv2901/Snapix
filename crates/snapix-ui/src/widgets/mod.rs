mod canvas;
mod geometry;
mod render;

use snapix_core::canvas::{Annotation, Document};

pub use canvas::DocumentCanvas;
pub(crate) use canvas::SharedColorButtons;
pub(crate) use geometry::composition_size;
pub(crate) use geometry::{
    layout_for_document, natural_image_bounds, paint_signature_preview_thumbnail,
};
pub(crate) use render::render_document_rgba;

#[derive(Clone, Copy)]
pub(super) struct CanvasLayout {
    pub(super) image_x: f64,
    pub(super) image_y: f64,
    pub(super) image_width: f64,
    pub(super) image_height: f64,
    pub(super) image_scale: f64,
    pub(super) viewport_x: f64,
    pub(super) viewport_y: f64,
    pub(super) viewport_width: f64,
    pub(super) viewport_height: f64,
}

#[derive(Clone, Copy)]
pub(super) enum CropInteractionMode {
    Move,
    ResizeTopLeft,
    ResizeTop,
    ResizeTopRight,
    ResizeLeft,
    ResizeRight,
    ResizeBottomLeft,
    ResizeBottom,
    ResizeBottomRight,
}

#[derive(Clone, Copy)]
pub(super) struct CropInteractionSession {
    pub(super) mode: CropInteractionMode,
    pub(super) initial_bounds: (f64, f64, f64, f64),
}

#[derive(Clone)]
pub(super) struct AnnotationMoveSession {
    pub(super) index: usize,
    pub(super) widget_start_x: f64,
    pub(super) widget_start_y: f64,
    pub(super) image_start_x: u32,
    pub(super) image_start_y: u32,
    pub(super) original: Annotation,
    pub(super) before_document: Document,
}

pub(super) const ANNOTATION_MOVE_THRESHOLD: f64 = 4.0;

#[derive(Clone)]
pub(super) struct AnnotationResizeSession {
    pub(super) index: usize,
    pub(super) mode: CropInteractionMode,
    pub(super) initial_bounds: (f64, f64, f64, f64),
    pub(super) original: Annotation,
    pub(super) before_document: Document,
}

#[derive(Clone)]
pub(super) struct ArrowResizeSession {
    pub(super) index: usize,
    pub(super) move_start: bool,
    pub(super) widget_start_x: f64,
    pub(super) widget_start_y: f64,
    pub(super) original: Annotation,
    pub(super) before_document: Document,
}

pub(crate) struct RenderedDocument {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[cfg(test)]
pub(crate) mod test_support {
    use snapix_core::canvas::{Document, Image};

    use super::CanvasLayout;

    pub(crate) fn sample_document(width: u32, height: u32) -> Document {
        Document::new(Image::new(
            width,
            height,
            vec![255; width as usize * height as usize * 4],
        ))
    }

    pub(crate) fn sample_layout() -> CanvasLayout {
        CanvasLayout {
            image_x: 10.0,
            image_y: 20.0,
            image_width: 200.0,
            image_height: 160.0,
            image_scale: 2.0,
            viewport_x: 10.0,
            viewport_y: 20.0,
            viewport_width: 200.0,
            viewport_height: 160.0,
        }
    }
}
