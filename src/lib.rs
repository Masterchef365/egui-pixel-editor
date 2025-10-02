#![allow(unused_imports)]
use std::{
    collections::{HashMap, HashSet},
    ops::RangeInclusive,
};

use egui::{
    epaint::ImageDelta, Color32, ColorImage, Event, EventFilter, Id, ImageData, Key, Modifiers,
    Painter, Pos2, Rect, Sense, Stroke, StrokeKind, TextureId, TextureOptions, Ui, Vec2, Widget,
};

mod tiled_image;

mod image;

mod image_editor;

mod undo;
mod brush;
mod ellipse;
mod widget;

pub use image_editor::ImageEditorState;
pub use brush::{Brush, BrushShape};
pub use widget::ImageEditor;
pub use image::{Image, ImageExt, Crop};
