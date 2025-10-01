use egui::{CentralPanel, Color32, ColorImage, DragValue, Rect, Scene};
use egui_pixel_editor::{Brush, BrushShape, ImageEditor, ImageEditorState};

fn main() {
    let mut image = ColorImage::filled([100, 100], Color32::BLACK);
    image.pixels.chunks_mut(3).for_each(|a| a[0] = Color32::RED);

    let mut color = Color32::WHITE;

    eframe::run_simple_native("image editor", Default::default(), move |ctx, _frame| {
        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Brush: ");
                ui.color_edit_button_srgba(&mut color);
            });
            ui.add(ImageEditor::new(&mut image, color));
        });
    })
    .unwrap();
}

