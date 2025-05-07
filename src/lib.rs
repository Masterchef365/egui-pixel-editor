use std::{collections::HashMap, ops::RangeInclusive};

use egui::{
    Color32, ColorImage, Id, ImageData, Painter, Pos2, Rect, TextureId, TextureOptions, Ui, Vec2,
};

pub trait Image {
    type Pixel: PixelInterface;
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
        (0..=self.width() as _, 0..=self.height() as _)
    }
}

impl PixelInterface for Color32 {
    fn as_rgba(&self) -> Color32 {
        *self
    }
}

pub struct ImageEditor<'image, T> {
    pub image: &'image dyn Image<Pixel = T>,
    /// Allow setting pixels outside of image_boundaries()?
    /// (could be used to e.g. expand the canvas dynamically)
    pub id_salt: Option<Id>,
    pub set_pixel_out_of_bounds: bool,
    pub is_interactive: bool,
}

impl<'image, T> ImageEditor<'image, T> {
    pub fn new(image: &'image dyn Image<Pixel = T>) -> Self {
        Self {
            image,
            id_salt: None,
            set_pixel_out_of_bounds: false,
            is_interactive: true,
        }
    }

    pub fn with_interactive(mut self, enable: bool) -> Self {
        self.is_interactive = enable;
        self
    }

    pub fn with_out_of_bounds_indexing(mut self, enable: bool) -> Self {
        self.set_pixel_out_of_bounds = enable;
        self
    }

    pub fn id_salt(mut self, id_salt: impl std::hash::Hash) -> Self {
        self.id_salt = Some(Id::new(id_salt));
        self
    }

    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
        todo!()
    }
}

struct ImageEditorImpl {
    tiles: HashMap<(isize, isize), (Rect, TextureId)>,
    texture_width: usize,
}

impl ImageEditorImpl {
    fn upload<T>(source: &dyn Image<Pixel = T>, ctx: &egui::Context) -> Self {
        const MAX_TEXTURE_SIZE: usize = 4096;

        let texture_width = ctx.fonts(|r| r.max_texture_side()).min(MAX_TEXTURE_SIZE);

        let mut tiles = HashMap::new();

        let (x_range, y_range) = source.image_boundaries();
        for y in y_range.step_by(MAX_TEXTURE_SIZE) {
            //let (_, y_range) = source.image_boundaries();
            let remain_y = (y_range.end() - y).min(MAX_TEXTURE_SIZE);
            for x in x_range.step_by(MAX_TEXTURE_SIZE) {
                //let (x_range, _) = source.image_boundaries();
                let remain_x = (x_range.end() - x).min(MAX_TEXTURE_SIZE);

                let rect = Rect::from_min_size(
                    Pos2::new(x as _, y as _),
                    Vec2::new(remain_x as _, remain_y as _),
                );
                let region = sample_to_image(source);

                let tex = ctx.tex_manager().write().alloc(
                    format!("Tile {x}, {y}"),
                    ImageData::Color(region.into()),
                    TextureOptions::NEAREST,
                );

                tiles.insert((x, y), (rect, tex));
            }
        }

        Self {
            tiles,
            texture_width,
        }
    }

    fn draw(&self, painter: &Painter) {
        let uv = Rect::from_min_size(Pos2::ZERO, Vec2::splat(1.));
        for (_, (rect, tex)) in &self.tiles {
            painter.image(*tex, *rect, uv, Color32::WHITE);
        }
    }
}

fn sample_to_image<T>(source: &dyn Image<Pixel = T>) -> ColorImage {
    todo!()
}

pub struct Crop<Pixel, I: Image<Pixel = Pixel>> {
    x_range: RangeInclusive<isize>,
    y_range: RangeInclusive<isize>,
    image: I,
}

impl<Pixel: PixelInterface, I: Image<Pixel = Pixel>> Image for Crop<Pixel, I>
{
    type Pixel = Pixel;
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
