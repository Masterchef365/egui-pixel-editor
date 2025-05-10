use std::{
    collections::{HashMap, HashSet},
    ops::RangeInclusive,
};

use egui::{
    epaint::ImageDelta, Color32, ColorImage, Event, EventFilter, Id, ImageData, Key, Modifiers,
    Painter, Pos2, Rect, Sense, Stroke, StrokeKind, TextureId, TextureOptions, Ui, Vec2, Widget,
};

use crate::image::{Image, ImageExt, PixelInterface};


#[derive(Copy, Clone)]
struct Tile {
    tex_id: TextureId,
    is_dirty: bool,
}

pub struct TiledEguiImage {
    tiles: HashMap<(isize, isize), Tile>,
    texture_width: usize,
}

impl TiledEguiImage {
    pub fn from_tile_size(texture_width: usize) -> Self {
        Self {
            tiles: Default::default(),
            texture_width,
        }
    }

    pub fn new(ctx: &egui::Context) -> Self {
        const MAX_TEXTURE_SIZE: usize = 512;
        let texture_width = ctx.fonts(|r| r.max_texture_side()).min(MAX_TEXTURE_SIZE);
        Self::from_tile_size(texture_width)
    }

    fn calc_tile(&self, x: isize, y: isize) -> (isize, isize) {
        let texture_width = self.texture_width as isize;
        (x / texture_width, y / texture_width)
    }

    pub fn notify_change(&mut self, x: isize, y: isize) {
        let tile_pos = self.calc_tile(x, y);
        if let Some(tile) = self.tiles.get_mut(&tile_pos) {
            tile.is_dirty = true;
        }
    }

    pub fn draw<T: PixelInterface>(
        &mut self,
        ui: &mut Ui,
        image: &mut impl Image<Pixel = T>,
        pos: Pos2,
    ) {
        let (x_range, y_range) = image.image_boundaries();
        let texture_width = self.texture_width as isize;

        // Draw and dynamically load tiles as the image bounds change
        for tile_y in y_range.start() / texture_width..=y_range.end() / texture_width {
            let y = tile_y * texture_width;
            for tile_x in x_range.start() / texture_width..=x_range.end() / texture_width {
                let x = tile_x * texture_width;

                let tile_rect =
                    Rect::from_min_size(Pos2::new(x as _, y as _), Vec2::splat(texture_width as _));

                let tile_rect = tile_rect.translate(pos.to_vec2());

                let mut get_patch = || {
                    let crop = image.crop(x..=x + texture_width - 1, y..=y + texture_width - 1);
                    sample_patch(&crop, self.texture_width)
                };

                let tex_options = TextureOptions::NEAREST;

                let tile = self.tiles.entry((tile_x, tile_y)).or_insert_with(|| {
                    let tex_id = ui.ctx().tex_manager().write().alloc(
                        format!("Tile {x}, {y}"),
                        get_patch().into(),
                        tex_options,
                    );
                    Tile::new(tex_id)
                });

                if tile.is_dirty {
                    let patch = get_patch();
                    ui.ctx()
                        .tex_manager()
                        .write()
                        .set(tile.tex_id, ImageDelta::full(patch, tex_options));
                    tile.is_dirty = false;
                }

                let uv = Rect::from_min_size(Pos2::ZERO, Vec2::splat(1.));
                ui.painter()
                    .image(tile.tex_id, tile_rect, uv, Color32::WHITE);
            }
        }
    }

    pub fn track<'tiles, 'image, I: Image>(
        &'tiles mut self,
        image: &'image mut I,
    ) -> TileChangeTracker<'image, 'tiles, I> {
        TileChangeTracker { image, tiles: self }
    }
}

impl Tile {
    pub fn new(tex_id: TextureId) -> Self {
        Self {
            tex_id,
            is_dirty: false,
        }
    }
}

pub struct TileChangeTracker<'image, 'tiles, I: Image + ?Sized> {
    image: &'image mut I,
    tiles: &'tiles mut TiledEguiImage,
}

impl<I> Image for TileChangeTracker<'_, '_, I>
where
    I: Image + ?Sized,
{
    type Pixel = I::Pixel;
    fn set_pixel(&mut self, x: isize, y: isize, px: Self::Pixel) {
        self.tiles.notify_change(x, y);
        self.image.set_pixel(x, y, px);
    }

    fn get_pixel(&self, x: isize, y: isize) -> Self::Pixel {
        self.image.get_pixel(x, y)
    }

    fn image_boundaries(&self) -> (RangeInclusive<isize>, RangeInclusive<isize>) {
        self.image.image_boundaries()
    }
}

fn sample_patch<T: PixelInterface>(
    source: &impl Image<Pixel = T>,
    texture_width: usize,
) -> ColorImage {
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
