use std::ops::Div;
use std::ops::RangeBounds;
use std::ops::Sub;

use bevy::math::IVec2;
use bevy::math::UVec2;
use bevy::prelude::Color;
use bevy::prelude::Component;
use bevy::prelude::Vec2;

use sark_grids::geometry::GridRect;
use sark_grids::grid::Side;
use sark_grids::Grid;
use sark_grids::GridPoint;
use sark_grids::Size2d;

use crate::border::Border;
use crate::fmt_tile::ColorFormat;
use crate::formatting::StringFormatter;
use crate::TileFormatter;

/// A simple terminal for writing text in a readable grid.
///
/// Contains various functions for drawing colorful text to a
/// terminal.
///
/// # Example
/// ```rust
/// use bevy_ascii_terminal::*;
/// use bevy::prelude::Color;
///
/// let mut term = Terminal::new([10,10]);
///
/// term.put_char([1,1], 'h'.fg(Color::RED));
/// term.put_string([2,1], "ello".bg(Color::BLUE));
///
/// let hello = term.get_string([1,1], 5);
/// ```
#[derive(Component, Clone, Debug, Default)]
pub struct Terminal {
    tiles: Grid<Tile>,
    size: UVec2,
    /// Tile to insert when a position is "cleared".
    ///
    /// The terminal will be filled with this tile when created.
    pub clear_tile: Tile,
    /// An optional border for the terminal.
    ///
    /// The terminal border is considered separate from the terminal itself,
    /// terminal positions and sizes do not include the border unless otherwise
    /// specified.
    border: Option<Border>,
}

/// A single tile of the terminal.
///
/// Defaults to a blank glyph with a black background and a white foreground.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tile {
    /// The glyph for the tile. Glyphs are mapped to sprites via the
    /// terminal's `UvMapping`
    pub glyph: char,
    /// The forergound color for the tile.
    pub fg_color: Color,
    /// The background color for the tile.
    pub bg_color: Color,
}

impl Tile {
    pub const DEFAULT_FGCOL: Color = Color::WHITE;
    pub const DEFAULT_BGCOL: Color = Color::BLACK;

    /// Create an invisible tile.
    pub fn transparent() -> Tile {
        Tile {
            glyph: ' ',
            fg_color: Color::srgba_u8(0, 0, 0, 0),
            bg_color: Color::srgba_u8(0, 0, 0, 0),
        }
    }
}

impl Default for Tile {
    fn default() -> Self {
        Tile {
            glyph: ' ',
            fg_color: Tile::DEFAULT_FGCOL,
            bg_color: Tile::DEFAULT_BGCOL,
        }
    }
}

impl From<char> for Tile {
    fn from(c: char) -> Self {
        Tile {
            glyph: c,
            ..Default::default()
        }
    }
}

impl Terminal {
    /// Construct a terminal with the given size
    pub fn new(size: impl Size2d) -> Terminal {
        let clear_tile = Tile::default();
        Terminal {
            tiles: Grid::new(size),
            size: size.as_uvec2(),
            clear_tile,
            ..Default::default()
        }
    }

    /// Specify a border for the terminal.
    ///
    /// The terminal border is considered separate from the terminal itself,
    /// writes and sizes within the terminal will ignore the border unless
    /// otherwise specified.
    pub fn with_border(mut self, border: Border) -> Self {
        self.border = Some(border);
        self
    }

    pub fn with_clear_tile(mut self, clear_tile: impl Into<Tile>) -> Self {
        self.clear_tile = clear_tile.into();
        self.clear();
        self
    }

    pub fn set_border(&mut self, border: Border) {
        self.border = Some(border);
    }

    pub fn remove_border(&mut self) {
        self.border = None;
    }

    pub fn border(&self) -> Option<&Border> {
        self.border.as_ref()
    }

    pub fn border_mut(&mut self) -> Option<&mut Border> {
        self.border.as_mut()
    }

    /// Resize the terminal.
    ///
    /// This will clear the terminal.
    pub fn resize(&mut self, size: impl Size2d) {
        self.tiles = Grid::new(size);
        self.size = size.as_uvec2();
    }

    /// The width of the terminal, excluding the border.
    pub fn width(&self) -> usize {
        self.size.x as usize
    }

    /// The height of the terminal, excluding the border.
    pub fn height(&self) -> usize {
        self.size.y as usize
    }

    /// The size of the terminal, excluding the border.
    pub fn size(&self) -> UVec2 {
        self.size
    }

    /// The size of the terminal, including the border if it has one.
    pub fn size_with_border(&self) -> UVec2 {
        let border_size = if self.has_border() {
            UVec2::splat(2)
        } else {
            UVec2::ZERO
        };
        self.size + border_size
    }

    pub fn width_with_border(&self) -> usize {
        if self.has_border() {
            self.width() + 2
        } else {
            self.width()
        }
    }

    pub fn height_with_border(&self) -> usize {
        if self.has_border() {
            self.height() + 2
        } else {
            self.height()
        }
    }

    /// Whether or not the terminal has a border.
    pub fn has_border(&self) -> bool {
        self.border.is_some()
    }

    /// Convert a local 2d position to it's corresponding
    /// 1d index
    #[inline]
    pub fn transform_lti(&self, xy: impl GridPoint) -> usize {
        self.tiles.transform_lti(xy)
    }

    /// Convert 1D index to it's local terminal position.
    #[inline]
    pub fn transform_itl(&self, i: usize) -> IVec2 {
        self.tiles.transform_itl(i)
    }

    /// Insert a formatted character into the terminal.
    ///
    /// The [`TileModifier`] trait allows you to optionally specify a foreground
    /// and/or background color for the tile using the `fg` and `bg` functions.
    /// If you don't specify a color then the existing color in the terminal tile will
    /// be unaffected.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bevy_ascii_terminal::prelude::*;
    /// use bevy::prelude::Color;
    ///
    /// let mut term = Terminal::new([10,10]);
    /// // Insert an 'a' with a blue foreground and a red background.
    /// term.put_char([2,3], 'a'.fg(Color::BLUE).bg(Color::RED));
    /// // Replace the 'a' with a 'q'. Foreground and background color will be
    /// // unaffected
    /// term.put_char([2,3], 'q');
    /// ```
    pub fn put_char(&mut self, xy: impl GridPoint, writer: impl TileFormatter) {
        let fmt = writer.format();
        fmt.draw(xy, self);
    }

    /// Change the foreground or background color for a single tile in the terminal.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bevy::prelude::*;
    /// use bevy_ascii_terminal::prelude::*;
    /// let mut term = Terminal::new([10,10]);
    ///
    /// // Set the background color for the given tile to blue.
    /// term.put_color([3,3], Color::BLUE.bg());
    /// ```
    pub fn put_color(&mut self, xy: impl GridPoint, color: ColorFormat) {
        let tile = self.get_tile_mut(xy);
        match color {
            ColorFormat::FgColor(col) => tile.fg_color = col,
            ColorFormat::BgColor(col) => tile.bg_color = col,
        }
    }

    /// Insert a [Tile].
    pub fn put_tile(&mut self, xy: impl GridPoint, tile: Tile) {
        let t = self.get_tile_mut(xy);
        *t = tile;
    }

    /// Write a formatted string to the terminal.
    ///
    /// The [`StringFormatter`] trait allows you to optionally specify a foreground
    /// and/or background color for the string using the `fg` and `bg` functions.
    /// If you don't specify a color then the existing colors in the terminal
    /// will be unaffected.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bevy_ascii_terminal::prelude::*;
    /// use bevy::prelude::Color;
    ///
    /// let mut term = Terminal::new([10,10]);
    /// // Write a blue "Hello" to the terminal
    /// term.put_string([1,2], "Hello".fg(Color::BLUE));
    /// // Write "Hello" with a green background
    /// term.put_string([2,1], "Hello".bg(Color::GREEN));
    /// ```
    ///
    /// You can also specify a `Pivot` for the string via the `pivot` function.
    /// This will align the string to the given pivot point. If the string
    /// is multiple lines, it will be adjusted appropriately to fit the given
    /// alignment.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bevy_ascii_terminal::*;
    /// use bevy::prelude::Color;
    ///
    /// let mut term = Terminal::new([10,10]);
    /// // Write a mutli-line string to the center of the terminal
    /// term.put_string([0,0].pivot(Pivot::Center), "Hello\nHow are you?");
    /// ```
    pub fn put_string<'a>(&mut self, xy: impl GridPoint, writer: impl StringFormatter<'a> + 'a) {
        let pivot = if let Some(pivot) = xy.get_pivot() {
            Vec2::from(pivot)
        } else {
            Vec2::ZERO
        };
        let origin = self.tiles.pivoted_point(xy);
        let fmt = writer.formatted();
        let string = &fmt.string;

        let h = string.lines().count() as i32;
        let y = (origin.y as f32 + (h - 1) as f32 * (1.0 - pivot.y)) as i32;

        let bounds = self.tiles.bounds();

        //println!("Origin {}, y {}", origin, y);

        for (i, line) in string.lines().enumerate() {
            let y = y - i as i32;
            //println!("Origin {}, Line {}. Bounds {}", origin, y, bounds);
            if y < bounds.min_i().y || y > bounds.max_i().y {
                break;
            }

            let len = line.chars().count().min(self.width());
            let x = origin.x - ((len - 1) as f32 * pivot.x) as i32;
            //println!("Getting index for {}, {}", x, y);
            let i = self.transform_lti([x, y]);
            //println!("X {}, I {}", x, i);
            let tiles = self.tiles.slice_mut()[i..].iter_mut().take(len);

            //println!("Writing string at {:?}", [x,y]);

            for (char, t) in line.chars().zip(tiles) {
                t.glyph = char;
                fmt.apply(t);
            }
        }
    }

    /// Clear a range of characters to the terminal's `clear_tile`.
    pub fn clear_string(&mut self, xy: impl GridPoint, len: usize) {
        let i = self.transform_lti(xy);
        for t in self.tiles.slice_mut()[i..].iter_mut().take(len) {
            *t = self.clear_tile;
        }
    }

    /// Retrieve the char from a tile.
    pub fn get_char(&self, xy: impl GridPoint) -> char {
        self.get_tile(xy).glyph
    }

    /// Retrieve a string from the terminal.
    pub fn get_string(&self, xy: impl GridPoint, len: usize) -> String {
        let i = self.transform_lti(xy);
        let iter = self.tiles.slice()[i..].iter().take(len).map(|t| t.glyph);

        String::from_iter(iter)
    }

    #[inline]
    /// Retrieve an immutable reference to a tile in the terminal.
    pub fn get_tile(&self, xy: impl GridPoint) -> &Tile {
        &self.tiles[self.transform_lti(xy)]
    }

    #[inline]
    /// Retrieve a mutable reference to a tile in the terminal.
    pub fn get_tile_mut(&mut self, xy: impl GridPoint) -> &mut Tile {
        let i = self.transform_lti(xy);
        &mut self.tiles[i]
    }

    /// Clear an area of the terminal to the terminal's `clear_tile`.
    pub fn clear_box(&mut self, xy: impl GridPoint, size: impl Size2d) {
        let [width, height] = size.as_array();
        let [x, y] = xy.as_array();
        for y in y..y + height as i32 {
            for x in x..x + width as i32 {
                self.put_tile([x, y], self.clear_tile);
            }
        }
    }

    /// Clear the terminal tiles to the terminal's `clear_tile`.
    pub fn clear(&mut self) {
        for t in self.tiles.iter_mut() {
            *t = self.clear_tile
        }
    }

    pub fn clear_line(&mut self, line: usize) {
        let tile = self.clear_tile;
        self.iter_row_mut(line).for_each(|t| *t = tile);
    }

    /// Returns true if the given position is inside the bounds of the terminal.
    #[inline]
    pub fn in_bounds(&self, xy: impl GridPoint) -> bool {
        self.tiles.in_bounds(xy)
    }

    /// An immutable iterator over the tiles of the terminal.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &Tile> {
        self.tiles.iter()
    }

    /// A mutable iterator over the tiles of the terminal.
    pub fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut Tile> {
        self.tiles.iter_mut()
    }

    /// An immutable iterator over an entire row of tiles in the terminal.
    pub fn iter_row(&self, y: usize) -> impl DoubleEndedIterator<Item = &Tile> {
        self.tiles.iter_row(y)
    }

    /// An immutable iterator over an entire row of tiles in the terminal.
    pub fn iter_row_mut(&mut self, y: usize) -> impl DoubleEndedIterator<Item = &mut Tile> {
        self.tiles.iter_row_mut(y)
    }

    /// An immutable iterator over a range of rows in the terminal.
    ///
    /// The iterator moves along each row from left to right, where 0 is the
    /// bottom row and `height - 1` is the top row.
    pub fn iter_rows(
        &self,
        range: impl RangeBounds<usize>,
    ) -> impl DoubleEndedIterator<Item = &[Tile]> {
        self.tiles.iter_rows(range)
    }

    /// A mutable iterator over a range of rows in the terminal.
    ///
    /// The iterator moves along each row from left to right, where 0 is the
    /// bottom row and `height - 1` is the top row.
    pub fn iter_rows_mut(
        &mut self,
        range: impl RangeBounds<usize>,
    ) -> impl DoubleEndedIterator<Item = &mut [Tile]> {
        self.tiles.iter_rows_mut(range)
    }

    /// An immutable iterator over an entire column of tiles in the terminal.
    ///
    /// The iterator moves from bottom to top.
    pub fn iter_column(&self, x: usize) -> impl DoubleEndedIterator<Item = &Tile> {
        self.tiles.iter_column(x)
    }

    /// A mutable iterator over an entire column of tiles in the terminal.
    ///
    /// The iterator moves from bottom to top.
    pub fn iter_column_mut(&mut self, x: usize) -> impl DoubleEndedIterator<Item = &mut Tile> {
        self.tiles.iter_column_mut(x)
    }

    /// Get the index for a given side on the terminal.
    pub fn side_index(&self, side: Side) -> usize {
        self.tiles.side_index(side)
    }

    /// Transform a position from terminal local space (origin bottom left) to
    /// world space (origin center).
    #[inline]
    pub fn transform_ltw(&self, pos: impl GridPoint) -> IVec2 {
        pos.as_ivec2() - self.size.as_ivec2().sub(1).div(2)
    }

    /// Transform a position from world space (origin center) to terminal local
    /// space (origin bottom left).
    #[inline]
    pub fn transform_wtl(&self, pos: impl GridPoint) -> IVec2 {
        //println!("P {}, Half size {}", pos.as_ivec2(),  self.size.as_ivec2().sub(1).div(2));
        pos.as_ivec2() + self.size.as_ivec2().div(2)
    }

    pub fn slice(&self) -> &[Tile] {
        self.tiles.slice()
    }

    pub fn slice_mut(&mut self) -> &mut [Tile] {
        self.tiles.slice_mut()
    }

    pub fn bounds_with_border(&self) -> GridRect {
        let bounds = self.bounds();
        if self.has_border() {
            bounds.resized([1, 1])
        } else {
            bounds
        }
    }

    pub fn bounds(&self) -> GridRect {
        let mut bounds = self.tiles.bounds();
        bounds.center -= self.size.as_ivec2() / 2;
        //println!("TERM BOUNDS {}", bounds);
        bounds
    }
}
 
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::color::palettes::basic::RED;

    #[test]
    fn put_char() {
        let mut term = Terminal::new([20, 20]);

        term.put_char([5, 5], 'h');

        assert_eq!('h', term.get_char([5, 5]));

        term.put_char([1, 2], 'q'.fg(Color::Srgba(RED)));

        let t = term.get_tile([1, 2]);
        assert_eq!('q', t.glyph);
        assert_eq!(Color::Srgba(RED), t.fg_color);
    }

    #[test]
    fn put_string() {
        let mut term = Terminal::new([20, 20]);
        // term.put_string([0, 0], "Hello");
        // assert_eq!("Hello", term.get_string([0, 0], 5));

        term.put_string([1, 1], "Hello");
        assert_eq!("He", term.get_string([1, 1], 2));
    }
}
