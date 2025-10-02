#![allow(unused_imports)]
use std::{
    collections::{HashMap, HashSet},
    ops::RangeInclusive,
};

use egui::{
    Color32, ColorImage, Event, EventFilter, Id, ImageData, Key, Modifiers, Painter, Pos2, Rect,
    Sense, Stroke, StrokeKind, TextureId, TextureOptions, Ui, Vec2, Widget, epaint::ImageDelta,
};

mod tiled_image;

mod image;

mod image_editor;

mod brush;
mod ellipse;
mod undo;
mod widget;

pub use brush::{Brush, BrushShape};
pub use image::{Crop, Image, ImageExt};
pub use image_editor::ImageEditorState;
pub use widget::ImageEditor;
