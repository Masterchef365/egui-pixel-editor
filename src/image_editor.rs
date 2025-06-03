use std::{
    collections::{HashMap, HashSet},
    ops::RangeInclusive,
};

use egui::{
    epaint::ImageDelta, Color32, ColorImage, Event, EventFilter, Id, ImageData, Key, Modifiers,
    Painter, Pos2, Rect, Sense, Stroke, StrokeKind, TextureId, TextureOptions, Ui, Vec2, Widget,
};

use crate::{
    image::{Image, ImageExt, PixelInterface},
    tiled_image::TiledEguiImage,
    undo::SparseImageUndoer,
    Brush,
};

pub struct ImageEditor<Pixel> {
    tiles: TiledEguiImage,
    undoer: SparseImageUndoer<Pixel>,
}

impl<Pixel: PixelInterface> ImageEditor<Pixel> {
    pub fn from_tile_size(tile_texture_width: usize) -> Self {
        Self {
            tiles: TiledEguiImage::from_tile_size(tile_texture_width),
            undoer: SparseImageUndoer::new(),
        }
    }

    pub fn new(ctx: &egui::Context) -> Self {
        Self {
            tiles: TiledEguiImage::new(ctx),
            undoer: SparseImageUndoer::new(),
        }
    }

    pub fn draw(&mut self, 
        ui: &mut Ui,
        image: &mut impl Image<Pixel = Pixel>,
        coloring_func: impl Fn(Pixel) -> Color32,
        pos: Pos2,
    ) {
        self.tiles.draw(ui, image, coloring_func, pos)
    }

    pub fn edit(
        &mut self,
        ui: &mut Ui,
        image: &mut impl Image<Pixel = Pixel>,
        coloring_func: impl Fn(Pixel) -> Color32,
        draw_color: Pixel,
        brush: Brush,
    ) -> egui::Response
    where
        Pixel: PartialEq + Copy,
    {
        let (x_range, y_range) = image.image_boundaries();
        let image_rect = Rect::from_min_max(
            Pos2::new(*x_range.start() as f32, *y_range.start() as f32),
            Pos2::new(*x_range.end() as f32 + 1.0, *y_range.end() as f32 + 1.0),
        );

        let resp = ui.allocate_response(image_rect.size(), Sense::click_and_drag());

        self.tiles.draw(ui, image, coloring_func, resp.rect.min);

        let mut image = self.tiles.track(image);

        let egui_to_pixel = |pos: Pos2| -> (isize, isize) {
            let pos = (pos - resp.rect.min.to_vec2()).floor();
            (pos.x as _, pos.y as _)
        };

        let pixel_to_egui =
            |(x, y): (isize, isize)| -> Pos2 { resp.rect.min + Vec2::new(x as _, y as _) };

        if let Some(pointer_pos) = resp.hover_pos() {
            let quantized_pos = pixel_to_egui(egui_to_pixel(pointer_pos));
            brush.shape.draw(ui.painter(), quantized_pos);
        }

        if let Some(interact_pointer_pos) = resp.interact_pointer_pos() {
            if resp.clicked() || resp.dragged() {
                let (xf, yf) = egui_to_pixel(interact_pointer_pos);
                let (xi, yi) = egui_to_pixel(interact_pointer_pos - resp.drag_delta());

                let mut image = self.undoer.track(&mut image);
                brush.pixels(xi, yi, xf, yf, |x, y| {
                    image.set_pixel_checked(x, y, draw_color);
                });
            }
        }

        if resp.drag_stopped() || resp.clicked() {
            self.undoer.new_frame();
        }

        let events = ui.input(|i| i.filtered_events(&EventFilter::default()));
        for event in events {
            match event {
                // Undo
                Event::Key {
                    key: Key::Z,
                    pressed: true,
                    modifiers,
                    ..
                } if modifiers.matches_logically(Modifiers::COMMAND) => {
                    self.undoer.undo(&mut image);
                }

                // Redo
                Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } if (modifiers.matches_logically(Modifiers::COMMAND) && key == Key::Y)
                    || (modifiers.matches_logically(Modifiers::SHIFT | Modifiers::COMMAND)
                        && key == Key::Z) =>
                {
                    self.undoer.redo(&mut image);
                }
                _ => (),
            }
        }

        resp
    }

    /// Forces the backend to upload to the GPU once more
    pub fn force_image_update(&mut self) {
        self.tiles.mark_all_dirty();
        self.undoer.reset();
    }
}
