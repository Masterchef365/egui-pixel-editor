use egui::{CentralPanel, Color32, ColorImage};
use egui_pixel_editor::ImageEditor;

fn main() {
    let mut image = ColorImage::new([100, 100], Color32::BROWN);
    eframe::run_simple_native("image editor", Default::default(), move |ctx, _frame| {
        CentralPanel::default().show(ctx, |ui| {
            ui.label("hi");
            ui.add(ImageEditor::new(&mut image));
        });
    }).unwrap();
}
