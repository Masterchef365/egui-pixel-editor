use egui::{Color32, Id, Ui, Widget};

use crate::{image::Image, Brush, ImageEditorState};

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
            brush_shape: Brush {
                shape: crate::BrushShape::Rectangle(1, 1),
                interpolate: true,
            },
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
        coloring_func: impl Fn(I::Pixel) -> Color32,
    ) -> egui::Response
    where
        I::Pixel: Send + Sync + PartialEq + Copy + 'static,
    {
        let id = self.get_id(ui);
        ImageEditorState::<I::Pixel>::load(ui.ctx(), id)
            .unwrap_or_default()
            .lock()
            .unwrap()
            .edit(
                ui,
                self.image,
                coloring_func,
                self.brush_value,
                self.brush_shape,
            )

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
    I::Pixel: Into<Color32> + Send + Sync + PartialEq + Copy + 'static,
{
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let id = self.get_id(ui);
        ImageEditorState::<I::Pixel>::load(ui.ctx(), id)
            .unwrap_or_default()
            .lock()
            .unwrap()
            .edit(
                ui,
                self.image,
                |px| px.into(),
                self.brush_value,
                self.brush_shape,
            )
    }
}
