use std::cell::RefCell;
use std::rc::Rc;

use anyhow::{Context, Result};
use gtk4::cairo;
use gtk4::prelude::*;
use snapix_core::canvas::{Annotation, Background, Color, Document, Image, TextStyle};

use crate::editor::{
    refresh_history_buttons, refresh_scope_label, CropDrag, CropSelection, EditorState, ToolKind,
};

pub(crate) struct RenderedDocument {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Clone, Copy)]
struct CanvasLayout {
    image_x: f64,
    image_y: f64,
    image_width: f64,
    image_height: f64,
    image_scale: f64,
}

#[derive(Clone, Copy)]
enum CropInteractionMode {
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
struct CropInteractionSession {
    mode: CropInteractionMode,
    initial_bounds: (f64, f64, f64, f64),
}

#[derive(Clone)]
pub struct DocumentCanvas {
    drawing_area: gtk4::DrawingArea,
}

impl DocumentCanvas {
    pub fn new(
        state: Rc<RefCell<EditorState>>,
        subtitle_label: gtk4::Label,
        scope_label: gtk4::Label,
        undo_button: gtk4::Button,
        redo_button: gtk4::Button,
    ) -> Self {
        let drawing_area = gtk4::DrawingArea::builder()
            .content_width(1100)
            .content_height(760)
            .hexpand(true)
            .vexpand(true)
            .focusable(true)
            .build();

        let draw_state = state.clone();
        drawing_area.set_draw_func(move |_area, cr, width, height| {
            let state = draw_state.borrow();
            if state.active_tool() == ToolKind::Crop {
                draw_crop_mode_canvas(cr, width, height, state.document());
                draw_crop_overlay(cr, &state, width, height);
            } else {
                draw_editor_canvas(cr, width, height, &state);
            }
        });

        let crop_interaction = Rc::new(RefCell::new(None::<CropInteractionSession>));
        let drag = gtk4::GestureDrag::new();
        {
            let state = state.clone();
            let drawing_area = drawing_area.clone();
            let scope_label = scope_label.clone();
            let crop_interaction = crop_interaction.clone();
            let undo_button = undo_button.clone();
            let redo_button = redo_button.clone();
            drag.connect_drag_begin(move |_gesture, x, y| {
                let width = drawing_area.allocated_width();
                let height = drawing_area.allocated_height();
                let mut state = state.borrow_mut();

                if state.active_tool() == ToolKind::Arrow {
                    let Some(layout) = preview_canvas_layout(state.document(), width, height)
                    else {
                        return;
                    };
                    if let Some((image_x, image_y)) =
                        widget_point_to_image_pixel(state.document(), layout, x, y)
                    {
                        state.begin_arrow_drag(x, y, image_x as f32, image_y as f32);
                        refresh_scope_label(&state, &scope_label);
                        refresh_history_buttons(&state, &undo_button, &redo_button);
                        drawing_area.grab_focus();
                        drawing_area.queue_draw();
                    }
                    return;
                }

                if state.active_tool() == ToolKind::Rectangle {
                    let Some(layout) = preview_canvas_layout(state.document(), width, height)
                    else {
                        return;
                    };
                    if point_in_layout(x, y, layout) {
                        state.begin_rect_drag(x, y);
                        refresh_scope_label(&state, &scope_label);
                        refresh_history_buttons(&state, &undo_button, &redo_button);
                        drawing_area.grab_focus();
                        drawing_area.queue_draw();
                    }
                    return;
                }

                if state.active_tool() == ToolKind::Ellipse {
                    let Some(layout) = preview_canvas_layout(state.document(), width, height)
                    else {
                        return;
                    };
                    if point_in_layout(x, y, layout) {
                        state.begin_ellipse_drag(x, y);
                        refresh_scope_label(&state, &scope_label);
                        refresh_history_buttons(&state, &undo_button, &redo_button);
                        drawing_area.grab_focus();
                        drawing_area.queue_draw();
                    }
                    return;
                }

                if state.active_tool() == ToolKind::Blur {
                    let Some(layout) = preview_canvas_layout(state.document(), width, height)
                    else {
                        return;
                    };
                    if point_in_layout(x, y, layout) {
                        state.begin_blur_drag(x, y);
                        refresh_scope_label(&state, &scope_label);
                        refresh_history_buttons(&state, &undo_button, &redo_button);
                        drawing_area.grab_focus();
                        drawing_area.queue_draw();
                    }
                    return;
                }

                if state.active_tool() != ToolKind::Crop {
                    return;
                }

                let Some(layout) = canvas_layout(state.document(), width, height) else {
                    return;
                };

                if let Some(selection) = state.crop_selection() {
                    if let Some(selection_bounds) = crop_selection_widget_bounds(layout, selection)
                    {
                        if let Some(mode) = hit_crop_interaction(selection_bounds, x, y) {
                            *crop_interaction.borrow_mut() = Some(CropInteractionSession {
                                mode,
                                initial_bounds: selection_bounds,
                            });
                            refresh_scope_label(&state, &scope_label);
                            drawing_area.grab_focus();
                            drawing_area.queue_draw();
                            return;
                        }
                    }
                }

                if point_in_layout(x, y, layout) {
                    state.begin_crop_drag(x, y);
                    *crop_interaction.borrow_mut() = None;
                    refresh_scope_label(&state, &scope_label);
                    refresh_history_buttons(&state, &undo_button, &redo_button);
                    drawing_area.grab_focus();
                    drawing_area.queue_draw();
                }
            });
        }
        {
            let state = state.clone();
            let drawing_area = drawing_area.clone();
            let crop_interaction = crop_interaction.clone();
            drag.connect_drag_update(move |_gesture, offset_x, offset_y| {
                let mut state = state.borrow_mut();

                if let Some(arrow_drag) = state.arrow_drag().cloned() {
                    let width = drawing_area.allocated_width();
                    let height = drawing_area.allocated_height();
                    if let Some(layout) = preview_canvas_layout(state.document(), width, height) {
                        if let Some((image_x, image_y)) = widget_point_to_image_pixel(
                            state.document(),
                            layout,
                            arrow_drag.widget_start_x() + offset_x,
                            arrow_drag.widget_start_y() + offset_y,
                        ) {
                            state.update_arrow_drag(image_x as f32, image_y as f32);
                        }
                    }
                } else if let Some(rect_drag) = state.rect_drag().cloned() {
                    state.update_rect_drag(
                        rect_drag.start_x() + offset_x,
                        rect_drag.start_y() + offset_y,
                    );
                } else if let Some(ellipse_drag) = state.ellipse_drag().cloned() {
                    state.update_ellipse_drag(
                        ellipse_drag.start_x() + offset_x,
                        ellipse_drag.start_y() + offset_y,
                    );
                } else if let Some(blur_drag) = state.blur_drag().cloned() {
                    state.update_blur_drag(
                        blur_drag.start_x() + offset_x,
                        blur_drag.start_y() + offset_y,
                    );
                } else if let Some(crop_drag) = state.crop_drag().cloned() {
                    state.update_crop_drag(
                        crop_drag.start_x() + offset_x,
                        crop_drag.start_y() + offset_y,
                    );
                } else if let Some(session) = *crop_interaction.borrow() {
                    let width = drawing_area.allocated_width();
                    let height = drawing_area.allocated_height();
                    if let Some(layout) = canvas_layout(state.document(), width, height) {
                        if let Some((x, y, crop_width, crop_height)) = adjusted_crop_bounds(
                            layout,
                            session.initial_bounds,
                            session.mode,
                            offset_x,
                            offset_y,
                        ) {
                            if let Some((image_x, image_y, image_width, image_height)) =
                                widget_rect_to_image_pixels(
                                    state.document(),
                                    layout,
                                    x,
                                    y,
                                    crop_width,
                                    crop_height,
                                )
                            {
                                state.set_crop_selection(
                                    image_x,
                                    image_y,
                                    image_width,
                                    image_height,
                                );
                            }
                        }
                    }
                } else {
                    return;
                }
                drawing_area.queue_draw();
            });
        }
        {
            let state = state.clone();
            let drawing_area = drawing_area.clone();
            let scope_label = scope_label.clone();
            let crop_interaction = crop_interaction.clone();
            let undo_button = undo_button.clone();
            let redo_button = redo_button.clone();
            drag.connect_drag_end(move |_gesture, offset_x, offset_y| {
                let width = drawing_area.allocated_width();
                let height = drawing_area.allocated_height();
                let mut state = state.borrow_mut();

                if let Some(arrow_drag) = state.arrow_drag().cloned() {
                    if let Some(layout) = preview_canvas_layout(state.document(), width, height) {
                        if let Some((image_x, image_y)) = widget_point_to_image_pixel(
                            state.document(),
                            layout,
                            arrow_drag.widget_start_x() + offset_x,
                            arrow_drag.widget_start_y() + offset_y,
                        ) {
                            state.update_arrow_drag(image_x as f32, image_y as f32);
                        }
                    }
                    state.commit_arrow_drag();
                    refresh_scope_label(&state, &scope_label);
                    refresh_history_buttons(&state, &undo_button, &redo_button);
                } else if let Some(rect_drag) = state.rect_drag().cloned() {
                    state.update_rect_drag(
                        rect_drag.start_x() + offset_x,
                        rect_drag.start_y() + offset_y,
                    );
                    if let Some(layout) = preview_canvas_layout(state.document(), width, height) {
                        if let Some((rect_x, rect_y, rect_width, rect_height)) =
                            crop_rect_to_image_pixels(state.document(), layout, &rect_drag)
                        {
                            state.commit_rect_annotation(rect_x, rect_y, rect_width, rect_height);
                            refresh_scope_label(&state, &scope_label);
                            refresh_history_buttons(&state, &undo_button, &redo_button);
                        } else {
                            state.clear_rect_drag();
                        }
                    } else {
                        state.clear_rect_drag();
                    }
                } else if let Some(ellipse_drag) = state.ellipse_drag().cloned() {
                    state.update_ellipse_drag(
                        ellipse_drag.start_x() + offset_x,
                        ellipse_drag.start_y() + offset_y,
                    );
                    if let Some(layout) = preview_canvas_layout(state.document(), width, height) {
                        if let Some((shape_x, shape_y, shape_width, shape_height)) =
                            crop_rect_to_image_pixels(state.document(), layout, &ellipse_drag)
                        {
                            state.commit_ellipse_annotation(
                                shape_x,
                                shape_y,
                                shape_width,
                                shape_height,
                            );
                            refresh_scope_label(&state, &scope_label);
                            refresh_history_buttons(&state, &undo_button, &redo_button);
                        } else {
                            state.clear_ellipse_drag();
                        }
                    } else {
                        state.clear_ellipse_drag();
                    }
                } else if let Some(blur_drag) = state.blur_drag().cloned() {
                    state.update_blur_drag(
                        blur_drag.start_x() + offset_x,
                        blur_drag.start_y() + offset_y,
                    );
                    if let Some(layout) = preview_canvas_layout(state.document(), width, height) {
                        if let Some((blur_x, blur_y, blur_width, blur_height)) =
                            crop_rect_to_image_pixels(state.document(), layout, &blur_drag)
                        {
                            state.commit_blur_rect(blur_x, blur_y, blur_width, blur_height);
                            refresh_scope_label(&state, &scope_label);
                            refresh_history_buttons(&state, &undo_button, &redo_button);
                        } else {
                            state.clear_blur_drag();
                        }
                    } else {
                        state.clear_blur_drag();
                    }
                } else if let Some(crop_drag) = state.crop_drag().cloned() {
                    state.update_crop_drag(
                        crop_drag.start_x() + offset_x,
                        crop_drag.start_y() + offset_y,
                    );
                    let final_crop_drag = state.crop_drag().cloned();

                    if let Some(layout) = canvas_layout(state.document(), width, height) {
                        if let Some((crop_x, crop_y, crop_width, crop_height)) =
                            final_crop_drag.as_ref().and_then(|crop_drag| {
                                crop_rect_to_image_pixels(state.document(), layout, crop_drag)
                            })
                        {
                            state.set_crop_selection(crop_x, crop_y, crop_width, crop_height);
                            refresh_scope_label(&state, &scope_label);
                            refresh_history_buttons(&state, &undo_button, &redo_button);
                        } else {
                            state.clear_crop_drag();
                            refresh_scope_label(&state, &scope_label);
                            refresh_history_buttons(&state, &undo_button, &redo_button);
                        }
                    } else {
                        state.clear_crop_drag();
                        refresh_scope_label(&state, &scope_label);
                        refresh_history_buttons(&state, &undo_button, &redo_button);
                    }
                }
                *crop_interaction.borrow_mut() = None;

                drawing_area.grab_focus();
                drawing_area.queue_draw();
            });
        }
        drawing_area.add_controller(drag);

        let click = gtk4::GestureClick::new();
        click.set_button(gtk4::gdk::BUTTON_PRIMARY);
        {
            let state = state.clone();
            let drawing_area = drawing_area.clone();
            let subtitle_label = subtitle_label.clone();
            let scope_label = scope_label.clone();
            let undo_button = undo_button.clone();
            let redo_button = redo_button.clone();
            click.connect_pressed(move |_gesture, _n_press, x, y| {
                let width = drawing_area.allocated_width();
                let height = drawing_area.allocated_height();
                let state_ref = state.borrow();
                if state_ref.active_tool() != ToolKind::Text {
                    return;
                }

                let Some(layout) = preview_canvas_layout(state_ref.document(), width, height)
                else {
                    return;
                };
                let Some((image_x, image_y)) =
                    widget_point_to_image_pixel(state_ref.document(), layout, x, y)
                else {
                    return;
                };
                drop(state_ref);

                let Some(root) = drawing_area.root() else {
                    return;
                };
                let Ok(window) = root.downcast::<gtk4::ApplicationWindow>() else {
                    return;
                };

                let dialog = gtk4::Dialog::builder()
                    .title("Add Text")
                    .transient_for(&window)
                    .modal(true)
                    .build();
                dialog.add_button("Cancel", gtk4::ResponseType::Cancel);
                dialog.add_button("Add", gtk4::ResponseType::Accept);
                dialog.set_default_response(gtk4::ResponseType::Accept);

                let content = dialog.content_area();
                content.set_spacing(10);
                content.set_margin_top(12);
                content.set_margin_bottom(12);
                content.set_margin_start(12);
                content.set_margin_end(12);

                let entry = gtk4::Entry::builder()
                    .placeholder_text("Type a short label")
                    .activates_default(true)
                    .build();
                content.append(
                    &gtk4::Label::builder()
                        .label("Text content")
                        .xalign(0.0)
                        .build(),
                );
                content.append(&entry);

                let state = state.clone();
                let drawing_area = drawing_area.clone();
                let subtitle_label = subtitle_label.clone();
                let scope_label = scope_label.clone();
                let undo_button = undo_button.clone();
                let redo_button = redo_button.clone();
                dialog.connect_response(move |dialog, response| {
                    if response == gtk4::ResponseType::Accept {
                        let mut state = state.borrow_mut();
                        if state.add_text_annotation(
                            image_x as f32,
                            image_y as f32,
                            entry.text().to_string(),
                        ) {
                            refresh_scope_label(&state, &scope_label);
                            refresh_history_buttons(&state, &undo_button, &redo_button);
                            crate::editor::refresh_subtitle(&state, &subtitle_label);
                            drawing_area.queue_draw();
                        }
                    }
                    dialog.close();
                });
                dialog.present();
            });
        }
        drawing_area.add_controller(click);

        Self { drawing_area }
    }

    pub fn widget(&self) -> &gtk4::DrawingArea {
        &self.drawing_area
    }

    pub fn refresh(&self) {
        self.drawing_area.queue_draw();
    }
}

pub(crate) fn render_document_rgba(document: &Document) -> Result<RenderedDocument> {
    let (width, height) = export_size(document);
    let mut surface = cairo::ImageSurface::create(cairo::Format::ARgb32, width, height)
        .context("Failed to create export surface")?;
    {
        let cr = cairo::Context::new(&surface).context("Failed to create cairo context")?;
        draw_canvas(&cr, width, height, document);
    }
    surface.flush();
    let stride = surface.stride() as usize;
    let data = surface
        .data()
        .context("Failed to read export surface data")?;
    let mut rgba = vec![0u8; (width * height * 4) as usize];

    for y in 0..height as usize {
        for x in 0..width as usize {
            let src = y * stride + x * 4;
            let dst = (y * width as usize + x) * 4;

            let b = data[src];
            let g = data[src + 1];
            let r = data[src + 2];
            let a = data[src + 3];

            if a == 0 {
                rgba[dst] = 0;
                rgba[dst + 1] = 0;
                rgba[dst + 2] = 0;
            } else {
                rgba[dst] = ((r as u16 * 255) / a as u16).min(255) as u8;
                rgba[dst + 1] = ((g as u16 * 255) / a as u16).min(255) as u8;
                rgba[dst + 2] = ((b as u16 * 255) / a as u16).min(255) as u8;
            }
            rgba[dst + 3] = a;
        }
    }

    Ok(RenderedDocument {
        width: width as u32,
        height: height as u32,
        rgba,
    })
}

fn export_size(document: &Document) -> (i32, i32) {
    const MIN_EXPORT_WIDTH: i32 = 1200;
    const MIN_EXPORT_HEIGHT: i32 = 800;
    const OUTER_MARGIN: i32 = 80;

    match document.base_image.as_ref() {
        Some(image) => {
            let padding = document.frame.padding.max(0.0).round() as i32;
            let width = image.width as i32 + padding * 2 + OUTER_MARGIN;
            let height = image.height as i32 + padding * 2 + OUTER_MARGIN;
            (width.max(MIN_EXPORT_WIDTH), height.max(MIN_EXPORT_HEIGHT))
        }
        None => (MIN_EXPORT_WIDTH, MIN_EXPORT_HEIGHT),
    }
}

fn draw_canvas(cr: &cairo::Context, width: i32, height: i32, document: &Document) {
    cr.set_source_rgb(0.09, 0.10, 0.13);
    cr.paint().ok();

    let margin = 40.0;
    let frame_x = margin;
    let frame_y = margin;
    let frame_w = (width as f64 - margin * 2.0).max(160.0);
    let frame_h = (height as f64 - margin * 2.0).max(160.0);

    paint_background(cr, frame_x, frame_y, frame_w, frame_h, &document.background);

    let image_bounds = inset_frame(
        frame_x,
        frame_y,
        frame_w,
        frame_h,
        document.frame.padding as f64,
    );

    if document.frame.shadow {
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.18);
        rounded_rect(
            cr,
            image_bounds.0 + 10.0,
            image_bounds.1 + 16.0,
            image_bounds.2,
            image_bounds.3,
            document.frame.corner_radius as f64,
        );
        cr.fill().ok();
    }

    if let Some(image) = document.base_image.as_ref() {
        paint_image(cr, image_bounds, image, document.frame.corner_radius as f64);
        if let Some(layout) = preview_canvas_layout(document, width, height) {
            draw_annotations(cr, document, layout);
        }
    } else {
        paint_empty_state(cr, image_bounds, document.frame.corner_radius as f64);
    }
}

fn draw_editor_canvas(cr: &cairo::Context, width: i32, height: i32, state: &EditorState) {
    draw_canvas(cr, width, height, state.document());
    if state.active_tool() == ToolKind::Arrow {
        if let Some(layout) = preview_canvas_layout(state.document(), width, height) {
            if let Some(arrow_drag) = state.arrow_drag() {
                draw_arrow(
                    cr,
                    layout,
                    arrow_drag.start_x(),
                    arrow_drag.start_y(),
                    arrow_drag.current_x(),
                    arrow_drag.current_y(),
                    &state.active_color(),
                    state.active_width(),
                );
            }
        }
    } else if state.active_tool() == ToolKind::Rectangle {
        if let Some(layout) = preview_canvas_layout(state.document(), width, height) {
            if let Some(rect_drag) = state.rect_drag() {
                draw_rect_preview(
                    cr,
                    layout,
                    rect_drag,
                    &state.active_color(),
                    state.active_width(),
                );
            }
        }
    } else if state.active_tool() == ToolKind::Ellipse {
        if let Some(layout) = preview_canvas_layout(state.document(), width, height) {
            if let Some(ellipse_drag) = state.ellipse_drag() {
                draw_ellipse_preview(
                    cr,
                    layout,
                    ellipse_drag,
                    &state.active_color(),
                    state.active_width(),
                );
            }
        }
    } else if state.active_tool() == ToolKind::Blur {
        if let Some(layout) = preview_canvas_layout(state.document(), width, height) {
            if let Some(blur_drag) = state.blur_drag() {
                draw_blur_preview(cr, layout, blur_drag);
            }
        }
    }
}

fn draw_annotations(cr: &cairo::Context, document: &Document, layout: CanvasLayout) {
    for annotation in &document.annotations {
        match annotation {
            Annotation::Arrow {
                from,
                to,
                color,
                width,
            } => draw_arrow(cr, layout, from.x, from.y, to.x, to.y, color, *width),
            Annotation::Text {
                pos,
                content,
                style,
            } => draw_text_annotation(cr, layout, pos.x, pos.y, content, style),
            Annotation::Rect {
                bounds,
                stroke,
                fill,
            } => draw_rect_annotation(
                cr,
                layout,
                bounds,
                &stroke.color,
                stroke.width,
                fill.as_ref(),
            ),
            Annotation::Ellipse {
                bounds,
                stroke,
                fill,
            } => draw_ellipse_annotation(
                cr,
                layout,
                bounds,
                &stroke.color,
                stroke.width,
                fill.as_ref(),
            ),
            Annotation::Blur { bounds, radius } => {
                draw_blur_annotation(cr, document, layout, bounds, *radius)
            }
            _ => {}
        }
    }
}

fn draw_blur_preview(cr: &cairo::Context, layout: CanvasLayout, blur_drag: &CropDrag) {
    let Some((x, y, width, height)) = crop_drag_widget_bounds(layout, blur_drag) else {
        return;
    };

    cr.save().ok();
    cr.set_source_rgba(1.0, 1.0, 1.0, 0.10);
    cr.rectangle(x, y, width, height);
    cr.fill().ok();

    cr.set_source_rgba(1.0, 1.0, 1.0, 0.9);
    cr.set_line_width(2.0);
    cr.rectangle(x, y, width, height);
    cr.stroke().ok();
    cr.restore().ok();
}

fn draw_rect_preview(
    cr: &cairo::Context,
    layout: CanvasLayout,
    rect_drag: &CropDrag,
    color: &Color,
    width: f32,
) {
    let Some((x, y, rect_width, rect_height)) = crop_drag_widget_bounds(layout, rect_drag) else {
        return;
    };
    draw_rect_shape(cr, x, y, rect_width, rect_height, color, width, None);
}

fn draw_rect_annotation(
    cr: &cairo::Context,
    layout: CanvasLayout,
    bounds: &snapix_core::canvas::Rect,
    color: &Color,
    width: f32,
    fill: Option<&Color>,
) {
    let x = layout.image_x + bounds.x as f64 * layout.image_scale;
    let y = layout.image_y + bounds.y as f64 * layout.image_scale;
    let rect_width = bounds.width as f64 * layout.image_scale;
    let rect_height = bounds.height as f64 * layout.image_scale;
    draw_rect_shape(cr, x, y, rect_width, rect_height, color, width, fill);
}

fn draw_ellipse_preview(
    cr: &cairo::Context,
    layout: CanvasLayout,
    ellipse_drag: &CropDrag,
    color: &Color,
    width: f32,
) {
    let Some((x, y, shape_width, shape_height)) = crop_drag_widget_bounds(layout, ellipse_drag)
    else {
        return;
    };
    draw_ellipse_shape(cr, x, y, shape_width, shape_height, color, width, None);
}

fn draw_ellipse_annotation(
    cr: &cairo::Context,
    layout: CanvasLayout,
    bounds: &snapix_core::canvas::Rect,
    color: &Color,
    width: f32,
    fill: Option<&Color>,
) {
    let x = layout.image_x + bounds.x as f64 * layout.image_scale;
    let y = layout.image_y + bounds.y as f64 * layout.image_scale;
    let shape_width = bounds.width as f64 * layout.image_scale;
    let shape_height = bounds.height as f64 * layout.image_scale;
    draw_ellipse_shape(cr, x, y, shape_width, shape_height, color, width, fill);
}

fn draw_rect_shape(
    cr: &cairo::Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    color: &Color,
    stroke_width: f32,
    fill: Option<&Color>,
) {
    cr.save().ok();
    if let Some(fill) = fill {
        set_color(cr, fill);
        cr.rectangle(x, y, width, height);
        cr.fill_preserve().ok();
    } else {
        cr.rectangle(x, y, width, height);
    }
    set_color(cr, color);
    cr.set_line_width(stroke_width as f64);
    cr.stroke().ok();
    cr.restore().ok();
}

fn draw_ellipse_shape(
    cr: &cairo::Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    color: &Color,
    stroke_width: f32,
    fill: Option<&Color>,
) {
    if width <= 0.0 || height <= 0.0 {
        return;
    }

    let rx = width / 2.0;
    let ry = height / 2.0;
    let cx = x + rx;
    let cy = y + ry;

    cr.save().ok();
    cr.translate(cx, cy);
    cr.scale(rx.max(1.0), ry.max(1.0));
    cr.arc(0.0, 0.0, 1.0, 0.0, std::f64::consts::TAU);
    cr.restore().ok();

    if let Some(fill) = fill {
        cr.save().ok();
        cr.translate(cx, cy);
        cr.scale(rx.max(1.0), ry.max(1.0));
        cr.arc(0.0, 0.0, 1.0, 0.0, std::f64::consts::TAU);
        set_color(cr, fill);
        cr.fill_preserve().ok();
        cr.restore().ok();
    }

    cr.save().ok();
    cr.translate(cx, cy);
    cr.scale(rx.max(1.0), ry.max(1.0));
    cr.arc(0.0, 0.0, 1.0, 0.0, std::f64::consts::TAU);
    set_color(cr, color);
    cr.set_line_width((stroke_width as f64 / rx.min(ry).max(1.0)).max(0.08));
    cr.stroke().ok();
    cr.restore().ok();
}

fn draw_blur_annotation(
    cr: &cairo::Context,
    document: &Document,
    layout: CanvasLayout,
    bounds: &snapix_core::canvas::Rect,
    radius: f32,
) {
    let Some(image) = document.base_image.as_ref() else {
        return;
    };

    let x = bounds.x.max(0.0).floor() as u32;
    let y = bounds.y.max(0.0).floor() as u32;
    let width = bounds.width.ceil().max(0.0) as u32;
    let height = bounds.height.ceil().max(0.0) as u32;
    if width < 2 || height < 2 || x >= image.width || y >= image.height {
        return;
    }

    let width = width.min(image.width - x);
    let height = height.min(image.height - y);
    let Some(region) = blurred_region_image(image, x, y, width, height, radius) else {
        return;
    };
    let Some(surface) = make_surface(&region) else {
        return;
    };

    let draw_x = layout.image_x + x as f64 * layout.image_scale;
    let draw_y = layout.image_y + y as f64 * layout.image_scale;
    let draw_w = width as f64 * layout.image_scale;
    let draw_h = height as f64 * layout.image_scale;

    cr.save().ok();
    cr.rectangle(draw_x, draw_y, draw_w, draw_h);
    cr.clip();
    cr.translate(draw_x, draw_y);
    cr.scale(layout.image_scale, layout.image_scale);
    cr.set_source_surface(&surface, 0.0, 0.0).ok();
    cr.paint().ok();
    cr.restore().ok();
}

fn draw_text_annotation(
    cr: &cairo::Context,
    layout: CanvasLayout,
    x: f32,
    y: f32,
    content: &str,
    style: &TextStyle,
) {
    let draw_x = layout.image_x + x as f64 * layout.image_scale;
    let draw_y = layout.image_y + y as f64 * layout.image_scale;
    let font_size = (style.font_size as f64 * layout.image_scale).max(14.0);

    cr.save().ok();
    cr.select_font_face(
        &style.font_family,
        cairo::FontSlant::Normal,
        if style.bold {
            cairo::FontWeight::Bold
        } else {
            cairo::FontWeight::Normal
        },
    );
    cr.set_font_size(font_size);

    cr.move_to(draw_x + 2.0, draw_y + 2.0);
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.45);
    cr.show_text(content).ok();

    cr.move_to(draw_x, draw_y);
    set_color(cr, &style.color);
    cr.show_text(content).ok();
    cr.restore().ok();
}

fn draw_arrow(
    cr: &cairo::Context,
    layout: CanvasLayout,
    from_x: f32,
    from_y: f32,
    to_x: f32,
    to_y: f32,
    color: &Color,
    width: f32,
) {
    let start_x = layout.image_x + from_x as f64 * layout.image_scale;
    let start_y = layout.image_y + from_y as f64 * layout.image_scale;
    let end_x = layout.image_x + to_x as f64 * layout.image_scale;
    let end_y = layout.image_y + to_y as f64 * layout.image_scale;

    let dx = end_x - start_x;
    let dy = end_y - start_y;
    let length = (dx * dx + dy * dy).sqrt();
    if length < 1.0 {
        return;
    }

    let angle = dy.atan2(dx);
    let head_length = (width as f64 * 3.4).max(14.0);
    let head_angle = 28.0_f64.to_radians();
    let stroke_width = width as f64;

    cr.save().ok();
    set_color(cr, color);
    cr.set_line_width(stroke_width);
    cr.set_line_cap(cairo::LineCap::Round);

    let shaft_end_x = end_x - head_length * angle.cos();
    let shaft_end_y = end_y - head_length * angle.sin();
    cr.move_to(start_x, start_y);
    cr.line_to(shaft_end_x, shaft_end_y);
    cr.stroke().ok();

    cr.move_to(end_x, end_y);
    cr.line_to(
        end_x - head_length * (angle - head_angle).cos(),
        end_y - head_length * (angle - head_angle).sin(),
    );
    cr.line_to(
        end_x - head_length * (angle + head_angle).cos(),
        end_y - head_length * (angle + head_angle).sin(),
    );
    cr.close_path();
    cr.fill().ok();
    cr.restore().ok();
}

fn draw_crop_mode_canvas(cr: &cairo::Context, width: i32, height: i32, document: &Document) {
    cr.set_source_rgb(0.07, 0.08, 0.10);
    cr.paint().ok();

    let Some(image) = document.base_image.as_ref() else {
        let margin = 40.0;
        let bounds = (
            margin,
            margin,
            (width as f64 - margin * 2.0).max(160.0),
            (height as f64 - margin * 2.0).max(160.0),
        );
        paint_empty_state(cr, bounds, 16.0);
        return;
    };

    let Some(layout) = canvas_layout(document, width, height) else {
        return;
    };

    cr.set_source_rgba(0.0, 0.0, 0.0, 0.22);
    rounded_rect(
        cr,
        layout.image_x + 12.0,
        layout.image_y + 18.0,
        layout.image_width,
        layout.image_height,
        18.0,
    );
    cr.fill().ok();

    paint_image(
        cr,
        (
            layout.image_x,
            layout.image_y,
            layout.image_width,
            layout.image_height,
        ),
        image,
        18.0,
    );
}

fn preview_canvas_layout(document: &Document, width: i32, height: i32) -> Option<CanvasLayout> {
    let image = document.base_image.as_ref()?;
    let margin = 40.0;
    let frame_x = margin;
    let frame_y = margin;
    let frame_w = (width as f64 - margin * 2.0).max(160.0);
    let frame_h = (height as f64 - margin * 2.0).max(160.0);
    let image_bounds = inset_frame(
        frame_x,
        frame_y,
        frame_w,
        frame_h,
        document.frame.padding as f64,
    );

    layout_for_bounds(image, image_bounds)
}

fn draw_crop_overlay(cr: &cairo::Context, state: &EditorState, width: i32, height: i32) {
    if state.active_tool() != ToolKind::Crop {
        return;
    }

    let Some(layout) = canvas_layout(state.document(), width, height) else {
        return;
    };
    let overlay = state
        .crop_drag()
        .and_then(|crop_drag| crop_drag_widget_bounds(layout, crop_drag))
        .or_else(|| {
            state
                .crop_selection()
                .and_then(|selection| crop_selection_widget_bounds(layout, selection))
        });
    let Some((x, y, overlay_width, overlay_height)) = overlay else {
        return;
    };
    let radius = 18.0;

    cr.save().ok();
    rounded_rect(
        cr,
        layout.image_x,
        layout.image_y,
        layout.image_width,
        layout.image_height,
        radius,
    );
    cr.clip();
    cr.set_source_rgba(0.02, 0.03, 0.04, 0.48);
    cr.set_fill_rule(cairo::FillRule::EvenOdd);
    cr.rectangle(
        layout.image_x,
        layout.image_y,
        layout.image_width,
        layout.image_height,
    );
    cr.rectangle(x, y, overlay_width, overlay_height);
    cr.fill().ok();
    cr.restore().ok();

    cr.save().ok();
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.34);
    cr.set_line_width(8.0);
    cr.rectangle(x, y, overlay_width, overlay_height);
    cr.stroke().ok();
    cr.restore().ok();

    cr.set_source_rgba(1.0, 1.0, 1.0, 0.92);
    cr.set_line_width(2.0);
    cr.rectangle(x, y, overlay_width, overlay_height);
    cr.stroke().ok();

    draw_crop_grid(cr, x, y, overlay_width, overlay_height);

    draw_crop_handle(cr, x, y);
    draw_crop_handle(cr, x + overlay_width / 2.0, y);
    draw_crop_handle(cr, x + overlay_width, y);
    draw_crop_handle(cr, x, y + overlay_height / 2.0);
    draw_crop_handle(cr, x + overlay_width, y + overlay_height / 2.0);
    draw_crop_handle(cr, x, y + overlay_height);
    draw_crop_handle(cr, x + overlay_width / 2.0, y + overlay_height);
    draw_crop_handle(cr, x + overlay_width, y + overlay_height);
}

fn draw_crop_grid(cr: &cairo::Context, x: f64, y: f64, width: f64, height: f64) {
    cr.save().ok();
    cr.set_source_rgba(1.0, 1.0, 1.0, 0.18);
    cr.set_line_width(1.0);

    let x_third = width / 3.0;
    let y_third = height / 3.0;

    for multiplier in [1.0, 2.0] {
        let vertical = x + x_third * multiplier;
        cr.move_to(vertical, y);
        cr.line_to(vertical, y + height);

        let horizontal = y + y_third * multiplier;
        cr.move_to(x, horizontal);
        cr.line_to(x + width, horizontal);
    }

    cr.stroke().ok();
    cr.restore().ok();
}

fn draw_crop_handle(cr: &cairo::Context, center_x: f64, center_y: f64) {
    const HANDLE_SIZE: f64 = 10.0;
    const HANDLE_RADIUS: f64 = 3.0;
    let half = HANDLE_SIZE / 2.0;

    cr.save().ok();
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.28);
    rounded_rect(
        cr,
        center_x - half,
        center_y - half,
        HANDLE_SIZE,
        HANDLE_SIZE,
        HANDLE_RADIUS,
    );
    cr.fill().ok();

    cr.set_source_rgb(1.0, 1.0, 1.0);
    rounded_rect(
        cr,
        center_x - half + 1.0,
        center_y - half + 1.0,
        HANDLE_SIZE - 2.0,
        HANDLE_SIZE - 2.0,
        HANDLE_RADIUS,
    );
    cr.fill().ok();
    cr.restore().ok();
}

fn hit_crop_interaction(
    bounds: (f64, f64, f64, f64),
    pointer_x: f64,
    pointer_y: f64,
) -> Option<CropInteractionMode> {
    let (x, y, width, height) = bounds;
    let center_x = x + width / 2.0;
    let center_y = y + height / 2.0;
    let right = x + width;
    let bottom = y + height;

    if near_handle(pointer_x, pointer_y, x, y) {
        return Some(CropInteractionMode::ResizeTopLeft);
    }
    if near_handle(pointer_x, pointer_y, center_x, y) {
        return Some(CropInteractionMode::ResizeTop);
    }
    if near_handle(pointer_x, pointer_y, right, y) {
        return Some(CropInteractionMode::ResizeTopRight);
    }
    if near_handle(pointer_x, pointer_y, x, center_y) {
        return Some(CropInteractionMode::ResizeLeft);
    }
    if near_handle(pointer_x, pointer_y, right, center_y) {
        return Some(CropInteractionMode::ResizeRight);
    }
    if near_handle(pointer_x, pointer_y, x, bottom) {
        return Some(CropInteractionMode::ResizeBottomLeft);
    }
    if near_handle(pointer_x, pointer_y, center_x, bottom) {
        return Some(CropInteractionMode::ResizeBottom);
    }
    if near_handle(pointer_x, pointer_y, right, bottom) {
        return Some(CropInteractionMode::ResizeBottomRight);
    }
    if pointer_x >= x && pointer_x <= right && pointer_y >= y && pointer_y <= bottom {
        return Some(CropInteractionMode::Move);
    }
    None
}

fn near_handle(pointer_x: f64, pointer_y: f64, handle_x: f64, handle_y: f64) -> bool {
    const HIT_RADIUS: f64 = 18.0;
    (pointer_x - handle_x).abs() <= HIT_RADIUS && (pointer_y - handle_y).abs() <= HIT_RADIUS
}

fn adjusted_crop_bounds(
    layout: CanvasLayout,
    initial_bounds: (f64, f64, f64, f64),
    mode: CropInteractionMode,
    offset_x: f64,
    offset_y: f64,
) -> Option<(f64, f64, f64, f64)> {
    const MIN_SIZE: f64 = 2.0;

    let (mut x, mut y, mut width, mut height) = initial_bounds;
    let left_limit = layout.image_x;
    let top_limit = layout.image_y;
    let right_limit = layout.image_x + layout.image_width;
    let bottom_limit = layout.image_y + layout.image_height;

    match mode {
        CropInteractionMode::Move => {
            x = (x + offset_x).clamp(left_limit, right_limit - width);
            y = (y + offset_y).clamp(top_limit, bottom_limit - height);
        }
        CropInteractionMode::ResizeTopLeft => {
            let new_left = (x + offset_x).clamp(left_limit, x + width - MIN_SIZE);
            let new_top = (y + offset_y).clamp(top_limit, y + height - MIN_SIZE);
            width += x - new_left;
            height += y - new_top;
            x = new_left;
            y = new_top;
        }
        CropInteractionMode::ResizeTop => {
            let new_top = (y + offset_y).clamp(top_limit, y + height - MIN_SIZE);
            height += y - new_top;
            y = new_top;
        }
        CropInteractionMode::ResizeTopRight => {
            let new_right = (x + width + offset_x).clamp(x + MIN_SIZE, right_limit);
            let new_top = (y + offset_y).clamp(top_limit, y + height - MIN_SIZE);
            width = new_right - x;
            height += y - new_top;
            y = new_top;
        }
        CropInteractionMode::ResizeLeft => {
            let new_left = (x + offset_x).clamp(left_limit, x + width - MIN_SIZE);
            width += x - new_left;
            x = new_left;
        }
        CropInteractionMode::ResizeRight => {
            let new_right = (x + width + offset_x).clamp(x + MIN_SIZE, right_limit);
            width = new_right - x;
        }
        CropInteractionMode::ResizeBottomLeft => {
            let new_left = (x + offset_x).clamp(left_limit, x + width - MIN_SIZE);
            let new_bottom = (y + height + offset_y).clamp(y + MIN_SIZE, bottom_limit);
            width += x - new_left;
            height = new_bottom - y;
            x = new_left;
        }
        CropInteractionMode::ResizeBottom => {
            let new_bottom = (y + height + offset_y).clamp(y + MIN_SIZE, bottom_limit);
            height = new_bottom - y;
        }
        CropInteractionMode::ResizeBottomRight => {
            let new_right = (x + width + offset_x).clamp(x + MIN_SIZE, right_limit);
            let new_bottom = (y + height + offset_y).clamp(y + MIN_SIZE, bottom_limit);
            width = new_right - x;
            height = new_bottom - y;
        }
    }

    if width < MIN_SIZE || height < MIN_SIZE {
        return None;
    }

    Some((x, y, width, height))
}

fn paint_background(
    cr: &cairo::Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    background: &Background,
) {
    match background {
        Background::Solid { color } => {
            set_color(cr, color);
        }
        Background::Gradient { from, to, .. } => {
            let gradient = cairo::LinearGradient::new(x, y, x + width, y + height);
            gradient.add_color_stop_rgba(0.0, to_f64(from.r), to_f64(from.g), to_f64(from.b), 1.0);
            gradient.add_color_stop_rgba(1.0, to_f64(to.r), to_f64(to.g), to_f64(to.b), 1.0);
            cr.set_source(&gradient).ok();
        }
        Background::Image { .. } | Background::BlurredScreenshot { .. } => {
            cr.set_source_rgb(0.15, 0.18, 0.23);
        }
    }

    rounded_rect(cr, x, y, width, height, 28.0);
    cr.fill().ok();
}

fn paint_empty_state(cr: &cairo::Context, bounds: (f64, f64, f64, f64), radius: f64) {
    let (x, y, width, height) = bounds;

    cr.set_source_rgb(0.96, 0.97, 0.99);
    rounded_rect(cr, x, y, width, height, radius);
    cr.fill().ok();

    cr.set_source_rgb(0.82, 0.85, 0.90);
    cr.set_line_width(2.0);
    rounded_rect(cr, x, y, width, height, radius);
    cr.stroke().ok();

    cr.set_source_rgb(0.73, 0.77, 0.83);
    cr.set_line_width(3.0);
    cr.move_to(x + width * 0.28, y + height * 0.32);
    cr.line_to(x + width * 0.72, y + height * 0.68);
    cr.move_to(x + width * 0.72, y + height * 0.32);
    cr.line_to(x + width * 0.28, y + height * 0.68);
    cr.stroke().ok();
}

fn paint_image(cr: &cairo::Context, bounds: (f64, f64, f64, f64), image: &Image, radius: f64) {
    let (x, y, max_width, max_height) = bounds;
    let image_w = image.width as f64;
    let image_h = image.height as f64;
    let scale = f64::min(max_width / image_w, max_height / image_h);
    let draw_w = image_w * scale;
    let draw_h = image_h * scale;
    let draw_x = x + (max_width - draw_w) / 2.0;
    let draw_y = y + (max_height - draw_h) / 2.0;

    rounded_rect(cr, draw_x, draw_y, draw_w, draw_h, radius);
    cr.clip();

    if let Some(surface) = make_surface(image) {
        cr.save().ok();
        cr.translate(draw_x, draw_y);
        cr.scale(scale, scale);
        cr.set_source_surface(&surface, 0.0, 0.0).ok();
        cr.paint().ok();
        cr.restore().ok();
    }

    cr.reset_clip();
}

fn make_surface(image: &Image) -> Option<cairo::ImageSurface> {
    let mut surface = cairo::ImageSurface::create(
        cairo::Format::ARgb32,
        image.width as i32,
        image.height as i32,
    )
    .ok()?;

    {
        let stride = surface.stride() as usize;
        let mut data = surface.data().ok()?;

        for y in 0..image.height as usize {
            for x in 0..image.width as usize {
                let src = (y * image.width as usize + x) * 4;
                let dst = y * stride + x * 4;

                let r = image.data[src];
                let g = image.data[src + 1];
                let b = image.data[src + 2];
                let a = image.data[src + 3];

                data[dst] = ((b as u16 * a as u16) / 255) as u8;
                data[dst + 1] = ((g as u16 * a as u16) / 255) as u8;
                data[dst + 2] = ((r as u16 * a as u16) / 255) as u8;
                data[dst + 3] = a;
            }
        }
    }

    surface.mark_dirty();
    Some(surface)
}

fn blurred_region_image(
    image: &Image,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    radius: f32,
) -> Option<Image> {
    if width == 0 || height == 0 {
        return None;
    }

    let mut sub = Vec::with_capacity((width * height * 4) as usize);
    for row in y..(y + height) {
        let start = ((row * image.width + x) * 4) as usize;
        let end = start + (width * 4) as usize;
        sub.extend_from_slice(&image.data[start..end]);
    }

    let rgba = image::RgbaImage::from_raw(width, height, sub)?;
    let blurred = image::imageops::blur(&rgba, radius.max(2.0));
    Some(Image::from_dynamic(image::DynamicImage::ImageRgba8(
        blurred,
    )))
}

fn canvas_layout(document: &Document, width: i32, height: i32) -> Option<CanvasLayout> {
    let stage_margin_x = 56.0;
    let stage_margin_y = 52.0;
    let x = stage_margin_x;
    let y = stage_margin_y;
    let max_width = (width as f64 - stage_margin_x * 2.0).max(160.0);
    let max_height = (height as f64 - stage_margin_y * 2.0).max(160.0);
    let image = document.base_image.as_ref()?;
    layout_for_bounds(image, (x, y, max_width, max_height))
}

fn layout_for_bounds(image: &Image, bounds: (f64, f64, f64, f64)) -> Option<CanvasLayout> {
    let (x, y, max_width, max_height) = bounds;
    let image_w = image.width as f64;
    let image_h = image.height as f64;
    if image_w <= 0.0 || image_h <= 0.0 {
        return None;
    }
    let scale = f64::min(max_width / image_w, max_height / image_h);
    let draw_w = image_w * scale;
    let draw_h = image_h * scale;
    let draw_x = x + (max_width - draw_w) / 2.0;
    let draw_y = y + (max_height - draw_h) / 2.0;

    Some(CanvasLayout {
        image_x: draw_x,
        image_y: draw_y,
        image_width: draw_w,
        image_height: draw_h,
        image_scale: scale,
    })
}

fn point_in_layout(x: f64, y: f64, layout: CanvasLayout) -> bool {
    x >= layout.image_x
        && y >= layout.image_y
        && x <= layout.image_x + layout.image_width
        && y <= layout.image_y + layout.image_height
}

fn widget_point_to_image_pixel(
    document: &Document,
    layout: CanvasLayout,
    x: f64,
    y: f64,
) -> Option<(u32, u32)> {
    let image = document.base_image.as_ref()?;
    if !point_in_layout(x, y, layout) {
        return None;
    }

    let image_x = ((x - layout.image_x) / layout.image_scale)
        .round()
        .clamp(0.0, image.width.saturating_sub(1) as f64) as u32;
    let image_y = ((y - layout.image_y) / layout.image_scale)
        .round()
        .clamp(0.0, image.height.saturating_sub(1) as f64) as u32;
    Some((image_x, image_y))
}

fn crop_drag_widget_bounds(
    layout: CanvasLayout,
    crop_drag: &CropDrag,
) -> Option<(f64, f64, f64, f64)> {
    let start_x = crop_drag
        .start_x()
        .clamp(layout.image_x, layout.image_x + layout.image_width);
    let start_y = crop_drag
        .start_y()
        .clamp(layout.image_y, layout.image_y + layout.image_height);
    let end_x = crop_drag
        .current_x()
        .clamp(layout.image_x, layout.image_x + layout.image_width);
    let end_y = crop_drag
        .current_y()
        .clamp(layout.image_y, layout.image_y + layout.image_height);

    let x = start_x.min(end_x);
    let y = start_y.min(end_y);
    let width = (start_x.max(end_x) - x).max(0.0);
    let height = (start_y.max(end_y) - y).max(0.0);

    if width < 2.0 || height < 2.0 {
        return None;
    }

    Some((x, y, width, height))
}

fn crop_selection_widget_bounds(
    layout: CanvasLayout,
    selection: CropSelection,
) -> Option<(f64, f64, f64, f64)> {
    let x = layout.image_x + selection.x() as f64 * layout.image_scale;
    let y = layout.image_y + selection.y() as f64 * layout.image_scale;
    let width = selection.width() as f64 * layout.image_scale;
    let height = selection.height() as f64 * layout.image_scale;

    if width < 2.0 || height < 2.0 {
        return None;
    }

    Some((x, y, width, height))
}

fn crop_rect_to_image_pixels(
    document: &Document,
    layout: CanvasLayout,
    crop_drag: &CropDrag,
) -> Option<(u32, u32, u32, u32)> {
    let (x, y, width, height) = crop_drag_widget_bounds(layout, crop_drag)?;
    widget_rect_to_image_pixels(document, layout, x, y, width, height)
}

fn widget_rect_to_image_pixels(
    document: &Document,
    layout: CanvasLayout,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Option<(u32, u32, u32, u32)> {
    let image = document.base_image.as_ref()?;
    let left = ((x - layout.image_x) / layout.image_scale).floor().max(0.0) as u32;
    let top = ((y - layout.image_y) / layout.image_scale).floor().max(0.0) as u32;
    let right = ((x + width - layout.image_x) / layout.image_scale)
        .ceil()
        .min(image.width as f64) as u32;
    let bottom = ((y + height - layout.image_y) / layout.image_scale)
        .ceil()
        .min(image.height as f64) as u32;

    if right <= left || bottom <= top {
        return None;
    }

    Some((left, top, right - left, bottom - top))
}

fn inset_frame(x: f64, y: f64, width: f64, height: f64, padding: f64) -> (f64, f64, f64, f64) {
    let padded_x = x + padding;
    let padded_y = y + padding;
    let padded_w = (width - padding * 2.0).max(80.0);
    let padded_h = (height - padding * 2.0).max(80.0);
    (padded_x, padded_y, padded_w, padded_h)
}

fn rounded_rect(cr: &cairo::Context, x: f64, y: f64, width: f64, height: f64, radius: f64) {
    let radius = radius.min(width / 2.0).min(height / 2.0);
    let degrees = std::f64::consts::PI / 180.0;

    cr.new_sub_path();
    cr.arc(
        x + width - radius,
        y + radius,
        radius,
        -90.0 * degrees,
        0.0 * degrees,
    );
    cr.arc(
        x + width - radius,
        y + height - radius,
        radius,
        0.0 * degrees,
        90.0 * degrees,
    );
    cr.arc(
        x + radius,
        y + height - radius,
        radius,
        90.0 * degrees,
        180.0 * degrees,
    );
    cr.arc(
        x + radius,
        y + radius,
        radius,
        180.0 * degrees,
        270.0 * degrees,
    );
    cr.close_path();
}

fn set_color(cr: &cairo::Context, color: &Color) {
    cr.set_source_rgba(
        to_f64(color.r),
        to_f64(color.g),
        to_f64(color.b),
        to_f64(color.a),
    );
}

fn to_f64(value: u8) -> f64 {
    f64::from(value) / 255.0
}
