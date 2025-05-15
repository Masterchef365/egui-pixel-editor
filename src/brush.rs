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
                let mut y = 0;

                let mirror = |v: Vec2| Vec2::new(v.x, -v.y+1.);

                let smart_line = |a: Vec2, b: Vec2| {
                    paint.line_segment([pos + a, pos + b], stroke);
                    paint.line_segment([pos + mirror(a), pos + mirror(b)], stroke);
                };

                for dx in -wx..=wx {
                    let ny = ellipse::solve_ellipse(wx, wy, dx) + 1;
                    let b = Vec2::new(dx as f32, ny as f32);
                    if y != ny {
                        let a = Vec2::new(dx as f32, y as f32);
                        smart_line(a, b);
                    }
                    let a = Vec2::new((dx + 1) as f32, ny as f32);
                    smart_line(a, b);

                    y = ny;
                }
                let ny = ellipse::solve_ellipse(wx, wy, wx) + 1;
                let a = Vec2::new((wx + 1) as f32, ny as f32);
                let b = Vec2::new((wx + 1) as f32, 0.0);
                smart_line(a, b);
            },
        }
    }
}

impl Default for Brush {
    fn default() -> Self {
        Self::Rectangle(0, 0)
    }
}
