use egui::{CentralPanel, Color32, ColorImage, DragValue, Rect, Scene};
use egui_pixel_editor::{Brush, BrushShape, ImageEditorState};

fn main() {
    let mut image = ColorImage::filled([1000, 1000], Color32::BLACK);
    image.pixels.chunks_mut(3).for_each(|a| a[0] = Color32::RED);

    let mut scene_rect = Rect::ZERO;

    let mut color = Color32::WHITE;

    let mut editor = None;

    let mut mode = false;
    let mut brush_width = 1_isize;
    let mut brush_height = 1_isize;
    let mut square_brush = false;

    let mut interpolate_brush = true;

    eframe::run_simple_native("image editor", Default::default(), move |ctx, _frame| {
        let editor = editor.get_or_insert_with(|| ImageEditorState::default());

        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Image Editor");
            ui.horizontal(|ui| {
                ui.label("Draw color: ");
                ui.color_edit_button_srgba(&mut color);

                ui.label("Brush mode");
                ui.selectable_value(&mut mode, false, "Ellipse");
                ui.selectable_value(&mut mode, true, "Rectangle");

                ui.label("Brush size");
                ui.add(DragValue::new(&mut brush_width).range(0..=isize::MAX));
                ui.add_enabled_ui(!square_brush, |ui| {
                    ui.label("x");
                    ui.add(DragValue::new(&mut brush_height).range(0..=isize::MAX));
                });

                ui.checkbox(&mut interpolate_brush, "Interpolate brush");

                ui.checkbox(&mut square_brush, "Square brush");
            });

            if square_brush {
                brush_height = brush_width;
            }

            let shape = match mode {
                false => BrushShape::Ellipse(brush_width, brush_height),
                true => BrushShape::Rectangle(brush_width, brush_height),
            };

            let brush = Brush {
                shape,
                interpolate: interpolate_brush,
            };

            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                Scene::new()
                    .zoom_range(0.1..=100.0)
                    .show(ui, &mut scene_rect, |ui| {
                        editor.edit(ui, &mut image, |c| c, color, brush);
                    });
            });
        });
    })
    .unwrap();
}
