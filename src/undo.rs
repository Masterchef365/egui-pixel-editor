use std::{
    collections::{HashMap, HashSet},
    ops::RangeInclusive,
};

use egui::{
    epaint::ImageDelta, Color32, ColorImage, Event, EventFilter, Id, ImageData, Key, Modifiers,
    Painter, Pos2, Rect, Sense, Stroke, StrokeKind, TextureId, TextureOptions, Ui, Vec2, Widget,
};

use crate::image::Image;

type UndoFrame<Pixel> = Vec<(isize, isize, Pixel, Pixel)>;

pub struct SparseImageUndoer<Pixel> {
    /// A list of "frames" of changes to the image.
    /// Each frame corresponds to a continuous mouse movement
    changes: Vec<UndoFrame<Pixel>>,
    redo: Vec<UndoFrame<Pixel>>,
    /// The maximum number of frames we keep before we start removing history
    pub max_frames: usize,
}

impl<Pixel> SparseImageUndoer<Pixel> {
    pub fn new() -> Self {
        Self {
            changes: vec![],
            redo: vec![],
            max_frames: 100,
        }
    }

    pub fn new_frame(&mut self) {
        self.changes.push(vec![]);
        if self.changes.len() > self.max_frames {
            self.changes.remove(0);
        }
    }

    pub fn set_pixel<I>(&mut self, image: &mut I, x: isize, y: isize, new_px: Pixel)
    where
        I: Image<Pixel = Pixel> + ?Sized,
        I::Pixel: PartialEq + Copy,
    {
        if self.changes.is_empty() {
            self.changes.push(Vec::new());
        }
        let frame = self.changes.last_mut().unwrap();

        let old_px = image.get_pixel(x, y);
        if new_px != old_px {
            frame.push((x, y, old_px, new_px));
            image.set_pixel(x, y, new_px);
            self.redo.clear();
        }
    }

    pub fn undo<I>(&mut self, image: &mut I)
    where
        I: Image<Pixel = Pixel> + ?Sized,
        I::Pixel: PartialEq + Copy,
    {
        let frame = loop {
            let Some(frame) = self.changes.pop() else {
                return;
            };
            if !frame.is_empty() {
                break frame;
            }
        };

        for (x, y, old, new) in frame.iter().rev().copied() {
            debug_assert!(
                new == image.get_pixel(x, y),
                "Undo History did not match canvas!"
            );
            image.set_pixel(x, y, old);
        }

        self.redo.push(frame);
    }

    pub fn redo<I>(&mut self, image: &mut I)
    where
        I: Image<Pixel = Pixel> + ?Sized,
        I::Pixel: PartialEq + Copy,
    {
        let Some(frame) = self.redo.pop() else {
            return;
        };

        for (x, y, old, new) in frame.iter().copied() {
            debug_assert!(
                old == image.get_pixel(x, y),
                "Redo History did not match canvas!"
            );
            image.set_pixel(x, y, new);
        }

        self.changes.push(frame);
    }

    pub fn track<'undoer, 'image, I: Image<Pixel = Pixel>>(
        &'undoer mut self,
        image: &'image mut I,
    ) -> UndoChangeTracker<'image, 'undoer, I> {
        UndoChangeTracker {
            image,
            undoer: self,
        }
    }
}

pub struct UndoChangeTracker<'image, 'undoer, I: Image + ?Sized> {
    image: &'image mut I,
    undoer: &'undoer mut SparseImageUndoer<I::Pixel>,
}

impl<I> Image for UndoChangeTracker<'_, '_, I>
where
    I: Image + ?Sized,
    I::Pixel: Copy + PartialEq,
{
    type Pixel = I::Pixel;
    fn set_pixel(&mut self, x: isize, y: isize, px: Self::Pixel) {
        self.undoer.set_pixel(self.image, x, y, px)
    }

    fn get_pixel(&self, x: isize, y: isize) -> Self::Pixel {
        self.image.get_pixel(x, y)
    }

    fn image_boundaries(&self) -> (RangeInclusive<isize>, RangeInclusive<isize>) {
        self.image.image_boundaries()
    }
}
