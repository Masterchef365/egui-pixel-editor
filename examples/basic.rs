use egui::{CentralPanel, Color32, ColorImage, Rect, Scene};
use egui_pixel_editor::ImageEditor;

fn main() {
    let mut image = ColorImage::new([10, 10], Color32::BROWN);
    image.pixels.chunks_mut(3).for_each(|a| a[0] = Color32::RED);

    let mut scene_rect = Rect::ZERO;

    let mut editor = None;
    eframe::run_simple_native("image editor", Default::default(), move |ctx, _frame| {
        let editor = editor.get_or_insert_with(|| ImageEditor::new(ctx));

        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Image Editor");
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                Scene::new().zoom_range(0.1..=100.0).show(ui, &mut scene_rect, |ui| {
                    editor.edit(ui, &mut image);
                });
            });
        });
    })
    .unwrap();
}
