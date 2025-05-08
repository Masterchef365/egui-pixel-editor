use std::{collections::HashMap, ops::RangeInclusive};

use egui::{
    Color32, ColorImage, Id, ImageData, Painter, Pos2, Rect, Sense, TextureId, TextureOptions, Ui,
    Vec2, Widget,
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

/*
pub struct ImageEditor<'image, T> {
    pub image: &'image dyn Image<Pixel = T>,
    /// Allow setting pixels outside of image_boundaries()?
    /// (could be used to e.g. expand the canvas dynamically)
    pub set_pixel_out_of_bounds: bool,
    pub id_salt: Option<Id>,
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
}

impl<'image, T> Widget for ImageEditor<'image, T> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let (width, height) = self.image.dimensions();
        let size = Vec2::new(width as f32, height as f32);
        let resp = ui.allocate_response(size, Sense::click_and_drag());

        let painter = ui.painter();
        painter.rect_filled(
            Rect::from_min_size(Pos2::ZERO, Vec2::splat(200.)),
            0.0,
            Color32::WHITE,
        );
        ui.ctx().data(|r| {}); 
        /*r.get_temp_mut_or_insert_with(self.id_salt, ImageEditorImpl::new));*/

        //self.just_draw(ui.painter(), resp);

        resp
    }
}
*/

pub struct ImageEditor {
    tiles: HashMap<(isize, isize), TextureId>,
    texture_width: usize,
}

impl ImageEditor {
    pub fn new(ctx: &egui::Context) -> Self {
        const MAX_TEXTURE_SIZE: usize = 4096;
        let texture_width = ctx.fonts(|r| r.max_texture_side()).min(MAX_TEXTURE_SIZE);
        Self {
            tiles: Default::default(),
            texture_width,
        }
    }

    pub fn edit<T: PixelInterface>(&mut self, ui: &mut Ui, image: &mut impl Image<Pixel = T>) {
        let (x_range, y_range) = image.image_boundaries();
        let image_rect = Rect::from_min_max(
            Pos2::new(*x_range.start() as f32, *y_range.start() as f32), 
            Pos2::new(*x_range.end() as f32, *y_range.end() as f32), 
        );

        let resp = ui.allocate_response(image_rect.size(), Sense::click_and_drag());

        self.draw(ui, image, resp.rect.min);
    }

    pub fn draw<T: PixelInterface>(&mut self, ui: &mut Ui, image: &mut impl Image<Pixel = T>, pos: Pos2) {
        let (x_range, y_range) = image.image_boundaries();
        let texture_width = self.texture_width as isize;

        // Draw and dynamically load tiles as the image bounds change
        for tile_y in y_range.start() / texture_width..=y_range.end() / texture_width {
            let y = tile_y * texture_width;
            for tile_x in x_range.start() / texture_width..=x_range.end() / texture_width {
                let x = tile_x * texture_width;

                let tile_rect = Rect::from_min_size(
                    Pos2::new(x as _, y as _),
                    Vec2::splat(texture_width as _),
                );

                let tile_rect = tile_rect.translate(pos.to_vec2());

                let tex_id = *self.tiles.entry((tile_x, tile_y)).or_insert_with(|| {
                    let crop = image.crop(x..=x + texture_width - 1, y..=y + texture_width - 1);
                    let region = sample_patch(&crop, self.texture_width);
                    ui.ctx().tex_manager().write().alloc(
                        format!("Tile {x}, {y}"),
                        region.into(),
                        TextureOptions::NEAREST,
                    )
                });

                let uv = Rect::from_min_size(Pos2::ZERO, Vec2::splat(1.));
                ui.painter().image(tex_id, tile_rect, uv, Color32::WHITE);
            }
        }
    }
}

fn sample_patch<T: PixelInterface>(source: &impl Image<Pixel = T>, texture_width: usize) -> ColorImage {
    let (x_range, y_range) = source.image_boundaries();
    let mut pixels = vec![];

    for y in 0..texture_width as isize {
        let y = y + y_range.start();
        for x in 0..texture_width as isize {
            let x = x + x_range.start();
            let color = match source.get_pixel_checked(x, y) {
                Some(px) => px.as_rgba(),
                None => Color32::TRANSPARENT,
            };
            pixels.push(color);
        }
    }

    ColorImage {
        size: [texture_width as usize; 2],
        pixels,
    }
}

pub struct Crop<'image, I: Image + ?Sized> {
    x_range: RangeInclusive<isize>,
    y_range: RangeInclusive<isize>,
    image: &'image mut I,
}

trait ImageExt: Image {
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
        let (x_range, y_range) = self.image_boundaries();
        (x_range.contains(&x) && y_range.contains(&y)).then(|| self.get_pixel(x, y))
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
