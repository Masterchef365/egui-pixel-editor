use egui::{CentralPanel, Color32, ColorImage, Rect, Scene};
use egui_pixel_editor::{Brush, ImageEditor};

fn main() {
    let mut image = ColorImage::new([1000, 1000], Color32::BLACK);
    image.pixels.chunks_mut(3).for_each(|a| a[0] = Color32::RED);

    let mut scene_rect = Rect::ZERO;

    let mut color = Color32::WHITE;

    let mut editor = None;

    let mut brush = Brush::Ellipse(5, 10);

    eframe::run_simple_native("image editor", Default::default(), move |ctx, _frame| {
        let editor = editor.get_or_insert_with(|| ImageEditor::new(ctx));

        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Image Editor");
            ui.horizontal(|ui| {
                ui.label("Draw color: ");
                ui.color_edit_button_srgba(&mut color);
            });
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                Scene::new().zoom_range(0.1..=100.0).show(ui, &mut scene_rect, |ui| {
                    editor.edit(ui, &mut image, color, brush);
                });
            });
        });
    })
    .unwrap();
}
