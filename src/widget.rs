use egui::{Color32, Id, Ui, Widget};

use crate::{image::Image, image_editor::ImageEditorState, Brush};

/// Image editor widget
pub struct ImageEditor<'image, I: Image> {
    image: &'image mut I,
    id_salt: Option<Id>,
    brush_value: I::Pixel,
    brush_shape: Brush,
    force_image_update: bool,
}

impl<'image, I: Image> ImageEditor<'image, I> {
    pub fn new(image: &'image mut I, brush_value: I::Pixel) -> Self {
        Self {
            image,
            id_salt: None,
            brush_shape: Brush {
                shape: crate::BrushShape::Rectangle(1, 1),
                interpolate: true,
            },
            brush_value,
            force_image_update: false,
        }
    }
}

impl<I: Image> ImageEditor<'_, I> {
    /// Edits an image, displaying it to the user with the `coloring_func`.
    /// When drawing, `brush_value` will be assigned to pixels, in the shape of `brush_shape`.
    pub fn edit(
        &mut self,
        ui: &mut Ui,
        coloring_func: impl Fn(I::Pixel) -> Color32,
    ) -> egui::Response
    where
        I::Pixel: Send + Sync + Copy + 'static,
    {
        let id = self.get_id(ui);

        let state = ImageEditorState::<I::Pixel>::load_or_default(ui.ctx(), id);
        let mut state = state.lock().unwrap();

        let resp = state.edit(
            ui,
            self.image,
            coloring_func,
            self.brush_value,
            self.brush_shape,
        );

        if self.force_image_update {
            state.force_image_update();
        }

        resp
    }

    /// Informs the backend that the image has been updated without any drawing having
    /// taken place.
    /// Currently, this dumps all undo/redo history.
    pub fn force_image_update(mut self, force: bool) -> Self {
        self.force_image_update = force;
        self
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

impl<I: Image> Widget for ImageEditor<'_, I>
where
    I::Pixel: Into<Color32> + Send + Sync + Copy + 'static,
{
    fn ui(mut self, ui: &mut Ui) -> egui::Response {
        self.edit(ui, |x| x.into())
    }
}
