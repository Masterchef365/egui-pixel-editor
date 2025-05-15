use std::{
    collections::{HashMap, HashSet},
    ops::RangeInclusive,
};

use egui::{
    epaint::ImageDelta, Color32, ColorImage, Event, EventFilter, Id, ImageData, Key, Modifiers,
    Painter, Pos2, Rect, Sense, Stroke, StrokeKind, TextureId, TextureOptions, Ui, Vec2, Widget,
};

use crate::ellipse;

#[derive(Copy, Clone)]
pub enum Brush {
    /// Width, Height
    Ellipse(isize, isize),
    /// Width, Height
    Rectangle(isize, isize),
}

impl Brush {
    pub fn pixels(&self, x: isize, y: isize, mut f: impl FnMut(isize, isize)) {
        match *self {
            Brush::Ellipse(wx, wy) => {
                for dy in -wy..=wy {
                    // Note: the ellipse is on its side here ...
                    let mx = ellipse::solve_ellipse(wy, wx, dy);

                    for dx in -mx..=mx {
                        f(x + dx, y + dy);
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

    pub fn draw(&self, paint: &Painter, pos: Pos2) {
        let stroke = Stroke::new(0.1, Color32::LIGHT_GRAY);
        match *self {
            Brush::Rectangle(wx, wy) => {
                let v = Vec2::new(wx as f32, wy as f32);
                let rect = Rect::from_min_max(pos - v, pos + v + Vec2::splat(1.0));
                paint.rect_stroke(
                    rect,
                    0.,
                    stroke,
                    StrokeKind::Middle,
                );
            },
            Brush::Ellipse(wx, wy) => {
                let x = pos.x as isize;
                let y = pos.y as isize;
                todo!()
            }
        }
    }
}

impl Default for Brush {
    fn default() -> Self {
        Self::Rectangle(0, 0)
    }
}
