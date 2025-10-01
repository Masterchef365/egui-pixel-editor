use std::{
    collections::{HashMap, HashSet}, marker::PhantomData, ops::RangeInclusive, sync::{Arc, Mutex}
};

use egui::{
    epaint::ImageDelta, Color32, ColorImage, Context, Event, EventFilter, Id, ImageData, Key, Modifiers, Painter, Pos2, Rect, Sense, Stroke, StrokeKind, TextureId, TextureOptions, Ui, Vec2, Widget
};

use crate::{
    image::{Image, ImageExt},
    tiled_image::TiledEguiImage,
    undo::SparseImageUndoer,
    Brush,
};

pub struct ImageEditorState<Pixel> {
    tiles: TiledEguiImage,
    undoer: SparseImageUndoer<Pixel>,
}

type SharedImageEditorState<Pixel> = Arc<Mutex<ImageEditorState<Pixel>>>;

impl<Pixel> Default for ImageEditorState<Pixel> {
    fn default() -> Self {
        Self {
            tiles: TiledEguiImage::from_tile_size(512),
            undoer: SparseImageUndoer::new(),
        }
    }
}

impl<Pixel: Send + Sync + 'static> ImageEditorState<Pixel> {
    /// Only draws the image, but doesn't allow editing it.
    pub fn draw(&mut self, 
        ui: &mut Ui,
        image: &mut impl Image<Pixel = Pixel>,
        coloring_func: impl Fn(Pixel) -> Color32,
        pos: Pos2,
    ) {
        self.tiles.draw(ui, image, coloring_func, pos)
    }

    /// Edits an image, displaying it to the user with the `coloring_func`.
    /// When drawing, `brush_value` will be assigned to pixels, in the shape of `brush_shape`.
    pub fn edit(
        &mut self,
        ui: &mut Ui,
        image: &mut impl Image<Pixel = Pixel>,
        coloring_func: impl Fn(Pixel) -> Color32,
        brush_value: Pixel,
        brush_shape: Brush,
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
            brush_shape.shape.draw(ui.painter(), quantized_pos);
        }

        if let Some(interact_pointer_pos) = resp.interact_pointer_pos() {
            if resp.clicked() || resp.dragged() {
                let (xf, yf) = egui_to_pixel(interact_pointer_pos);
                let (xi, yi) = egui_to_pixel(interact_pointer_pos - resp.drag_delta());

                let mut image = self.undoer.track(&mut image);
                brush_shape.pixels(xi, yi, xf, yf, |x, y| {
                    image.set_pixel_checked(x, y, brush_value);
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

    /// Informs the backend that the image has been updated without any drawing having
    /// taken place.
    /// Currently, this dumps all undo/redo history.
    pub fn force_image_update(&mut self) {
        self.tiles.mark_all_dirty();
        self.undoer.reset();
    }

    pub fn load(ctx: &Context, id: Id) -> Option<SharedImageEditorState<Pixel>> {
        ctx.data_mut(|d| d.get_temp(id))
    }

    pub fn store(value: SharedImageEditorState<Pixel>, ctx: &Context, id: Id) {
        ctx.data_mut(|d| d.insert_temp(id, value));
    }
}

/*
pub struct ImageEditor<'image, I: Image> {
    image: &'image mut I,
    id_salt: Option<Id>,
    brush_value: I::Pixel,
    brush_shape: Brush,
}

impl<'image, I: Image> ImageEditor<'image, I> {
    pub fn new(image: &'image mut I, brush_value: I::Pixel) -> Self {
        Self {
            image,
            id_salt: None,
            brush_shape: Brush { shape: crate::BrushShape::Rectangle(1, 1), interpolate: true },
            brush_value,
        }
    }
}

impl<I: Image> ImageEditor<'_, I> {
    /// Edits an image, displaying it to the user with the `coloring_func`.
    /// When drawing, `brush_value` will be assigned to pixels, in the shape of `brush_shape`.
    pub fn edit(
        &mut self,
        ui: &mut Ui,
        image: &mut I,
        coloring_func: impl Fn(I::Pixel) -> Color32,
    ) -> egui::Response
    where
        I::Pixel: PartialEq + Copy,
    {
        //let id = self.get_id(ui);
        //let mut state = ImageEditorState::<Pixel>::load(ui.ctx(), id).unwrap_or_default();
        todo!()
    }

    pub fn id_salt(mut self, id_salt: impl std::hash::Hash) -> Self {
        self.id_salt = Some(Id::new(id_salt));
        self
    }

    pub fn with_brush(mut self, brush: Brush) -> Self {
        self.brush_shape = brush;
        self
    }

    fn get_id(&self, ui: &mut Ui) -> Id {
        if let Some(id_salt) = self.id_salt {
            ui.make_persistent_id(id_salt)
        } else {
            ui.next_auto_id()
        }
    }
}

impl<I: Image> Widget for ImageEditor<'_, I> where I::Pixel: Into<Color32> {
    fn ui(mut self, ui: &mut Ui) -> egui::Response {
        let id = self.get_id(ui);
        let mut state = ImageEditorState::<I::Pixel>::load(ui.ctx(), id).unwrap_or_default();
        state.lock().unwrap().edit(ui, self.image, |px| px.into(), self.brush_value, self.brush_shape)
    }
}
*/
