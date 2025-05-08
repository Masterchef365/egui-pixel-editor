use egui::{CentralPanel, Color32, ColorImage};
use egui_pixel_editor::ImageEditor;

fn main() {
    let mut image = ColorImage::new([100, 100], Color32::BROWN);
    image.pixels.chunks_mut(3).for_each(|a| a[0] = Color32::RED);

    let mut editor = None;
    eframe::run_simple_native("image editor", Default::default(), move |ctx, _frame| {
        let editor = editor.get_or_insert_with(|| ImageEditor::new(ctx));

        CentralPanel::default().show(ctx, |ui| {
            ui.label("hi");
            editor.edit(ui, &mut image);
        });
    })
    .unwrap();
}
