use std::{
    collections::{HashMap, HashSet},
    ops::RangeInclusive,
};

use egui::{
    epaint::ImageDelta, Color32, ColorImage, Event, EventFilter, Id, ImageData, Key, Modifiers,
    Painter, Pos2, Rect, Sense, Stroke, StrokeKind, TextureId, TextureOptions, Ui, Vec2, Widget,
};

pub trait Image {
    type Pixel;
    /// Gets the pixel at `(x, y)`
    /// Allowed to panic outside of image_boundaries if `set_pixel_out_of_bounds` is `false`.
    fn get_pixel(&self, x: isize, y: isize) -> Self::Pixel;
    /// Sets the pixel at `(x, y)` to `px`.
    /// Allowed to panic outside of image_boundaries if `set_pixel_out_of_bounds` is `false`.
    fn set_pixel(&mut self, x: isize, y: isize, px: Self::Pixel);
    /// Returns the boundaries of the image; may grow over time (but not shrink!)
    fn image_boundaries(&self) -> (RangeInclusive<isize>, RangeInclusive<isize>);
}

pub trait PixelInterface {
    /// What color should we display this pixel as?
    /// Allows transparency.
    /// This should be a pure function.
    fn as_rgba(&self) -> Color32;
}

pub struct Crop<'image, I: Image + ?Sized> {
    x_range: RangeInclusive<isize>,
    y_range: RangeInclusive<isize>,
    image: &'image mut I,
}

pub trait ImageExt: Image {
    fn crop(
        &mut self,
        x_range: RangeInclusive<isize>,
        y_range: RangeInclusive<isize>,
    ) -> Crop<Self> {
        let (image_x_range, image_y_range) = self.image_boundaries();
        let x_range = (*x_range.start()).max(*image_x_range.start())
            ..=(*x_range.end()).min(*image_x_range.end());
        let y_range = (*y_range.start()).max(*image_y_range.start())
            ..=(*y_range.end()).min(*image_y_range.end());
        Crop {
            x_range,
            y_range,
            image: self,
        }
    }

    fn dimensions(&self) -> (usize, usize) {
        let (x_range, y_range) = self.image_boundaries();
        let width: usize = (x_range.end() - x_range.start() + 1)
            .try_into()
            .expect("Invalid width range");
        let height: usize = (y_range.end() - y_range.start() + 1)
            .try_into()
            .expect("Invalid height range");
        (width, height)
    }

    fn get_pixel_checked(&self, x: isize, y: isize) -> Option<Self::Pixel> {
        self.bounds_check(x, y).then(|| self.get_pixel(x, y))
    }

    fn set_pixel_checked(&mut self, x: isize, y: isize, px: Self::Pixel) -> bool {
        let ret = self.bounds_check(x, y);
        if ret {
            self.set_pixel(x, y, px);
        } 
        ret
    }

    fn bounds_check(&self, x: isize, y: isize) -> bool {
        let (x_range, y_range) = self.image_boundaries();
        x_range.contains(&x) && y_range.contains(&y)
    }
}

impl<T: Image + ?Sized> ImageExt for T {}

impl<I: Image> Image for Crop<'_, I> {
    type Pixel = I::Pixel;
    fn get_pixel(&self, x: isize, y: isize) -> Self::Pixel {
        assert!(
            self.x_range.contains(&x) && self.y_range.contains(&y),
            "Out of bounds get pixel in crop at {}, {} not in {:?}, {:?}",
            x,
            y,
            self.x_range,
            self.y_range
        );
        self.image.get_pixel(x, y)
    }

    fn set_pixel(&mut self, x: isize, y: isize, px: Self::Pixel) {
        assert!(
            self.x_range.contains(&x) && self.y_range.contains(&y),
            "Out of bounds set pixel in crop at {}, {} not in {:?}, {:?}",
            x,
            y,
            self.x_range,
            self.y_range
        );
        self.image.set_pixel(x, y, px);
    }

    fn image_boundaries(&self) -> (RangeInclusive<isize>, RangeInclusive<isize>) {
        (self.x_range.clone(), self.y_range.clone())
    }
}

impl Image for ColorImage {
    type Pixel = Color32;
    fn get_pixel(&self, x: isize, y: isize) -> Self::Pixel {
        let x: usize = x.try_into().unwrap();
        let y: usize = y.try_into().unwrap();
        self.pixels[x + y * self.width()]
    }

    fn set_pixel(&mut self, x: isize, y: isize, px: Self::Pixel) {
        let x: usize = x.try_into().unwrap();
        let y: usize = y.try_into().unwrap();
        let width = self.width();
        self.pixels[x + y * width] = px;
    }

    fn image_boundaries(&self) -> (RangeInclusive<isize>, RangeInclusive<isize>) {
        (0..=(self.width() - 1) as _, 0..=(self.height() - 1) as _)
    }
}

impl PixelInterface for Color32 {
    fn as_rgba(&self) -> Color32 {
        *self
    }
}


