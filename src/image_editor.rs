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
};

#[derive(Copy, Clone)]
pub enum Brush {
    /// Width, Height
    Ellipse(isize, isize),
    /// Width, Height
    Rectangle(isize, isize),
}

pub struct ImageEditor<Pixel> {
    tiles: TiledEguiImage,
    undoer: SparseImageUndoer<Pixel>,
}

impl<Pixel: PixelInterface> ImageEditor<Pixel> {
    pub fn new(ctx: &egui::Context) -> Self {
        Self {
            tiles: TiledEguiImage::new(ctx),
            undoer: SparseImageUndoer::new(),
        }
    }

    pub fn edit(
        &mut self,
        ui: &mut Ui,
        image: &mut impl Image<Pixel = Pixel>,
        draw_color: Pixel,
        brush: Brush,
    ) where
        Pixel: PartialEq + Copy,
    {
        let (x_range, y_range) = image.image_boundaries();
        let image_rect = Rect::from_min_max(
            Pos2::new(*x_range.start() as f32, *y_range.start() as f32),
            Pos2::new(*x_range.end() as f32 + 1.0, *y_range.end() as f32 + 1.0),
        );

        let resp = ui.allocate_response(image_rect.size(), Sense::click_and_drag());

        if resp.drag_started() || resp.clicked() {
            self.undoer.new_frame();
        }

        self.tiles.draw(ui, image, resp.rect.min);

        let mut image = self.tiles.track(image);

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

        let egui_to_pixel = |pos: Pos2| -> (isize, isize) {
            let pos = (pos - resp.rect.min.to_vec2()).floor();
            (pos.x as _, pos.y as _)
        };

        let pixel_to_egui =
            |(x, y): (isize, isize)| -> Pos2 { resp.rect.min + Vec2::new(x as _, y as _) };

        if let Some(pointer_pos) = resp.hover_pos() {
            let (x, y) = egui_to_pixel(pointer_pos);
            let rect = Rect::from_min_max(pixel_to_egui((x, y)), pixel_to_egui((x + 1, y + 1)));
            ui.painter().rect_stroke(
                rect,
                0.,
                Stroke::new(0.1, Color32::LIGHT_GRAY),
                StrokeKind::Middle,
            );
        }

        if let Some(interact_pointer_pos) = resp.interact_pointer_pos() {
            let (x, y) = egui_to_pixel(interact_pointer_pos);
            let mut image = self.undoer.track(&mut image);
            brush.pixels(x, y, |x, y| {
                image.set_pixel_checked(x, y, draw_color);
            });
            //self.undoer.sync_set_pixel(image, x, y, draw);
        }
    }
}

impl Brush {
    fn pixels(&self, x: isize, y: isize, mut f: impl FnMut(isize, isize)) {
        match *self {
            Brush::Ellipse(wx, wy) => {
                for dy in -wy..=wy {
                    for dx in -wx..=wx {
                        let dx2 = dx * dx;
                        let dy2 = dy * dy;
                        let wx2 = wx * wx;
                        let wy2 = wy * wy;
                        if dy2 * wx2 < wy2 * wx2 - wy2 * dx2 {
                            f(x + dx, y + dy);
                        }
                    }
                }
            }
            Brush::Rectangle(wx, wy) => {
                for dy in -wy..=wy {
                    for dx in -wx..=wx {
                        f(x + dx, y + dy);
                    }
                }
            }
        }
    }
}
