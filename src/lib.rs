#![allow(unused_imports)]
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

#[derive(Copy, Clone)]
struct Tile {
    tex_id: TextureId,
    is_dirty: bool,
}

type UndoFrame<Pixel> = Vec<(isize, isize, Pixel, Pixel)>;

struct SparseImageUndoer<Pixel> {
    /// A list of "frames" of changes to the image.
    /// Each frame corresponds to a continuous mouse movement
    changes: Vec<UndoFrame<Pixel>>,
    redo: Vec<UndoFrame<Pixel>>,
}

impl<Pixel> SparseImageUndoer<Pixel> {
    pub fn new() -> Self {
        Self {
            changes: vec![],
            redo: vec![],
        }
    }

    pub fn new_frame(&mut self) {
        self.changes.push(vec![]);
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
        dbg!(frame.len());
        if new_px != old_px {
            frame.push((x, y, old_px, new_px));
            image.set_pixel(x, y, new_px);
        }
    }

    pub fn undo<I>(&mut self, image: &mut I)
    where
        I: Image<Pixel = Pixel> + ?Sized,
        I::Pixel: PartialEq + Copy,
    {
        let Some(frame) = self.changes.pop() else {
            return;
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

        self.redo.push(frame);
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

pub struct TiledEguiImage {
    tiles: HashMap<(isize, isize), Tile>,
    texture_width: usize,
}

pub struct ImageEditor<Pixel> {
    image: TiledEguiImage,
    undoer: SparseImageUndoer<Pixel>,
}

impl TiledEguiImage {
    pub fn new(ctx: &egui::Context) -> Self {
        const MAX_TEXTURE_SIZE: usize = 512;
        let texture_width = ctx.fonts(|r| r.max_texture_side()).min(MAX_TEXTURE_SIZE);
        Self {
            tiles: Default::default(),
            texture_width,
        }
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

impl<Pixel: PixelInterface> ImageEditor<Pixel> {
    pub fn new(ctx: &egui::Context) -> Self {
        Self {
            image: TiledEguiImage::new(ctx),
            undoer: SparseImageUndoer::new(),
        }
    }

    pub fn edit(&mut self, ui: &mut Ui, image: &mut impl Image<Pixel = Pixel>, draw: Pixel)
    where
        Pixel: PartialEq + Copy,
    {
        let (x_range, y_range) = image.image_boundaries();
        let image_rect = Rect::from_min_max(
            Pos2::new(*x_range.start() as f32, *y_range.start() as f32),
            Pos2::new(*x_range.end() as f32 + 1.0, *y_range.end() as f32 + 1.0),
        );

        let resp = ui.allocate_response(image_rect.size(), Sense::click_and_drag());

        if resp.drag_started() || resp.clicked() {
            self.undoer.new_frame();
        }

        let events = ui.input(|i| i.filtered_events(&EventFilter::default()));
        for event in events {
            match event {
                // Undo
                Event::Key {
                    key: Key::Z,
                    pressed: true,
                    modifiers,
                    ..
                } if modifiers.matches_logically(Modifiers::COMMAND) => {
                    eprintln!("UNDO EVENT");
                    self.undoer.undo(image);
                }

                // Redo
                Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } if (modifiers.matches_logically(Modifiers::COMMAND) && key == Key::Y)
                    || (modifiers.matches_logically(Modifiers::SHIFT | Modifiers::COMMAND)
                        && key == Key::Z) =>
                {
                    eprintln!("REDO EVENT");
                    self.undoer.redo(image);
                }
                _ => (),
            }
        }

        let egui_to_pixel = |pos: Pos2| -> (isize, isize) {
            let pos = (pos - resp.rect.min.to_vec2()).floor();
            (pos.x as _, pos.y as _)
        };

        let pixel_to_egui =
            |(x, y): (isize, isize)| -> Pos2 { resp.rect.min + Vec2::new(x as _, y as _) };

        self.image.draw(ui, image, resp.rect.min);

        if let Some(pointer_pos) = resp.hover_pos() {
            let (x, y) = egui_to_pixel(pointer_pos);
            let rect = Rect::from_min_max(pixel_to_egui((x, y)), pixel_to_egui((x + 1, y + 1)));
            ui.painter().rect_stroke(
                rect,
                0.,
                Stroke::new(0.1, Color32::LIGHT_GRAY),
                StrokeKind::Middle,
            );
        }

        if let Some(interact_pointer_pos) = resp.interact_pointer_pos() {
            let (x, y) = egui_to_pixel(interact_pointer_pos);
            let mut image = self.image.track(image);
            let mut image = self.undoer.track(&mut image);
            image.set_pixel(x, y, draw);
            //self.undoer.sync_set_pixel(image, x, y, draw);
        }
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
