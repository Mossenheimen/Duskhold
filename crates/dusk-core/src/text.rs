//! The text grid: how buildings are written (prefabs), how tests draw their maps, and how the
//! tools print the world. Rows alternate:
//!
//! - even rows are edges: `+` corners (or spaces) between horizontal edges;
//! - odd rows are cells: vertical edges between cells, so cells sit in the odd columns.
//!
//! On an edge, `-` (horizontal) or `|` (vertical) is a wall, `D` a shut door, `d` an open one,
//! `W` a shut window, `w` an open one, `f` a fence, and a space is open. A cell's character
//! comes from the legend.
//!
//! ```text
//! +-+-+-+-+
//! |. .|. .|
//! +   +D+-+
//! |. . . .d
//! +-+W+-+-+
//! ```

use std::fmt;

use crate::geom::TilePos;
use crate::grid::{Edge, Grid, Ground, Tile};

/// What one cell's character stands for in the default legend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub tile: Tile,
    /// A letter or `@` on the cell, such as a spawn point.
    pub marker: Option<char>,
}

/// The default legend: `.` a floor (indoors), `,` grass, `:` dirt, `=` road, `~` water, `#`
/// rock (or furniture), a space nothing at all. A letter or `@` marks a floor tile (tests and the
/// demo use `@` for the character and `z` for a shambler).
pub fn default_legend(ch: char) -> Option<Cell> {
    let outdoors = |ground| Tile {
        ground,
        ..Tile::GRASS
    };
    let tile = match ch {
        '.' => Tile::FLOOR,
        ',' => Tile::GRASS,
        ':' => outdoors(Ground::Dirt),
        '=' => outdoors(Ground::Road),
        '~' => outdoors(Ground::Water),
        '#' => Tile::ROCK,
        ' ' => Tile::VOID,
        c if c == '@' || c.is_ascii_alphabetic() => {
            return Some(Cell {
                tile: Tile::FLOOR,
                marker: Some(c),
            });
        }
        _ => return None,
    };
    Some(Cell { tile, marker: None })
}

/// A character the parser didn't expect, with its line and column (both from 1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextError {
    pub line: usize,
    pub column: usize,
    pub found: char,
    pub expected: &'static str,
}

impl fmt::Display for TextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "line {}, column {}: found {:?}, expected {}",
            self.line, self.column, self.found, self.expected
        )
    }
}

impl std::error::Error for TextError {}

/// A parsed text grid: cells of any kind (`C`, from the legend) and the edges between them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextGrid<C> {
    pub width: i32,
    pub height: i32,
    /// `width × height`, row by row.
    cells: Vec<C>,
    /// `width × (height + 1)`: row `height` is the south side.
    north: Vec<Edge>,
    /// `(width + 1) × height`: column `width` is the east side.
    west: Vec<Edge>,
}

impl<C: Copy> TextGrid<C> {
    pub fn parse(text: &str, legend: impl Fn(char) -> Option<C>) -> Result<TextGrid<C>, TextError> {
        let mut lines: Vec<Vec<char>> = text.lines().map(|line| line.chars().collect()).collect();
        while lines
            .first()
            .is_some_and(|line| line.iter().all(|c| c.is_whitespace()))
        {
            lines.remove(0);
        }
        while lines
            .last()
            .is_some_and(|line| line.iter().all(|c| c.is_whitespace()))
        {
            lines.pop();
        }
        let rows = lines.len();
        let cols = lines.iter().map(Vec::len).max().unwrap_or(0);
        if rows < 3 || rows.is_multiple_of(2) || cols < 3 {
            return Err(TextError {
                line: 1,
                column: 1,
                found: ' ',
                expected: "an odd number of rows (edges, cells, edges, ...), at least 3 by 3",
            });
        }
        let (width, height) = ((cols - 1) / 2, (rows - 1) / 2);
        let at = |r: usize, c: usize| lines[r].get(c).copied().unwrap_or(' ');
        let err = |r: usize, c: usize, expected: &'static str| TextError {
            line: r + 1,
            column: c + 1,
            found: at(r, c),
            expected,
        };

        let mut cells = Vec::with_capacity(width * height);
        for cy in 0..height {
            for cx in 0..width {
                let (r, c) = (2 * cy + 1, 2 * cx + 1);
                cells.push(legend(at(r, c)).ok_or_else(|| err(r, c, "a cell from the legend"))?);
            }
        }
        let mut north = Vec::with_capacity(width * (height + 1));
        for cy in 0..=height {
            let r = 2 * cy;
            for cx in 0..width {
                let c = 2 * cx + 1;
                north.push(edge_from(at(r, c), '-').ok_or_else(|| {
                    err(r, c, "an edge: '-', 'D', 'd', 'W', 'w', 'f' or a space")
                })?);
            }
            for cx in 0..=width {
                if !matches!(at(r, 2 * cx), '+' | ' ') {
                    return Err(err(r, 2 * cx, "a corner: '+' or a space"));
                }
            }
        }
        let mut west = Vec::with_capacity((width + 1) * height);
        for cy in 0..height {
            let r = 2 * cy + 1;
            for cx in 0..=width {
                let c = 2 * cx;
                west.push(edge_from(at(r, c), '|').ok_or_else(|| {
                    err(r, c, "an edge: '|', 'D', 'd', 'W', 'w', 'f' or a space")
                })?);
            }
        }
        Ok(TextGrid {
            width: width as i32,
            height: height as i32,
            cells,
            north,
            west,
        })
    }

    pub fn cell(&self, x: i32, y: i32) -> C {
        self.cells[(y * self.width + x) as usize]
    }

    /// The edge on a cell's north side; `y == height` is the south side of the last row.
    pub fn north(&self, x: i32, y: i32) -> Edge {
        self.north[(y * self.width + x) as usize]
    }

    /// The edge on a cell's west side; `x == width` is the east side of the last column.
    pub fn west(&self, x: i32, y: i32) -> Edge {
        self.west[(y * (self.width + 1) + x) as usize]
    }

    /// Turned a quarter turn clockwise: what faced north faces east.
    pub fn rotated(&self) -> TextGrid<C> {
        let (w, h) = (self.width, self.height);
        // The new grid is h wide and w tall; its cell (x, y) was the old cell (y, h - 1 - x).
        let mut cells = Vec::with_capacity(self.cells.len());
        let mut north = Vec::with_capacity(h as usize * (w as usize + 1));
        let mut west = Vec::with_capacity((h as usize + 1) * w as usize);
        for y in 0..w {
            for x in 0..h {
                cells.push(self.cell(y, h - 1 - x));
            }
        }
        // An old west edge is a new north edge, and an old north edge a new west edge.
        for y in 0..=w {
            for x in 0..h {
                north.push(self.west(y, h - 1 - x));
            }
        }
        for y in 0..w {
            for x in 0..=h {
                west.push(self.north(y, h - x));
            }
        }
        TextGrid {
            width: h,
            height: w,
            cells,
            north,
            west,
        }
    }

    /// Flipped east to west.
    pub fn mirrored(&self) -> TextGrid<C> {
        let (w, h) = (self.width, self.height);
        let cells = (0..h)
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            .map(|(x, y)| self.cell(w - 1 - x, y))
            .collect();
        let north = (0..=h)
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            .map(|(x, y)| self.north(w - 1 - x, y))
            .collect();
        let west = (0..h)
            .flat_map(|y| (0..=w).map(move |x| (x, y)))
            .map(|(x, y)| self.west(w - x, y))
            .collect();
        TextGrid {
            width: w,
            height: h,
            cells,
            north,
            west,
        }
    }

    /// Draws the grid back as text, with each cell's character from `symbol`.
    pub fn render(&self, symbol: impl Fn(C) -> char) -> String {
        let mut out = String::new();
        for y in 0..=self.height {
            for x in 0..=self.width {
                let edges = [
                    (x > 0).then(|| self.north(x - 1, y)),
                    (x < self.width).then(|| self.north(x, y)),
                    (y > 0).then(|| self.west(x, y - 1)),
                    (y < self.height).then(|| self.west(x, y)),
                ];
                let walled = edges.iter().flatten().any(|&e| e != Edge::Open);
                out.push(if walled { '+' } else { ' ' });
                if x < self.width {
                    out.push(edge_symbol(self.north(x, y), '-'));
                }
            }
            out.push('\n');
            if y == self.height {
                break;
            }
            for x in 0..=self.width {
                out.push(edge_symbol(self.west(x, y), '|'));
                if x < self.width {
                    out.push(symbol(self.cell(x, y)));
                }
            }
            out.push('\n');
        }
        out
    }
}

impl TextGrid<Cell> {
    /// Writes the grid into the world, its north-west cell at `origin`.
    pub fn stamp(&self, grid: &mut Grid, origin: TilePos) {
        let (w, h) = (self.width, self.height);
        for cy in 0..h {
            for cx in 0..w {
                grid.set_tile(origin.offset(cx, cy), self.cell(cx, cy).tile);
            }
        }
        for cy in 0..=h {
            for cx in 0..w {
                grid.set_north(origin.offset(cx, cy), self.north(cx, cy));
            }
        }
        for cy in 0..h {
            for cx in 0..=w {
                grid.set_west(origin.offset(cx, cy), self.west(cx, cy));
            }
        }
    }

    /// Each marker with its cell, in reading order.
    pub fn markers(&self) -> Vec<(char, i32, i32)> {
        let mut out = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                if let Some(m) = self.cell(x, y).marker {
                    out.push((m, x, y));
                }
            }
        }
        out
    }
}

fn edge_from(ch: char, wall: char) -> Option<Edge> {
    Some(match ch {
        ' ' => Edge::Open,
        'D' => Edge::door(false),
        'd' => Edge::door(true),
        'W' => Edge::window(false),
        'w' => Edge::window(true),
        'f' => Edge::Fence,
        c if c == wall => Edge::Wall,
        _ => return None,
    })
}

fn edge_symbol(edge: Edge, wall: char) -> char {
    match edge {
        Edge::Open => ' ',
        Edge::Wall => wall,
        // Boards don't show in a text grid.
        Edge::Door { open: false, .. } => 'D',
        Edge::Door { open: true, .. } => 'd',
        Edge::Window { open: false, .. } => 'W',
        Edge::Window { open: true, .. } => 'w',
        Edge::Fence => 'f',
    }
}

/// A tile's character in the default legend (fixtures show as `#` when solid).
pub fn tile_symbol(tile: Tile) -> char {
    if tile.solid {
        return '#';
    }
    match tile.ground {
        Ground::Void => ' ',
        Ground::Grass => ',',
        Ground::Dirt => ':',
        Ground::Road => '=',
        Ground::Water => '~',
        Ground::Floor => '.',
    }
}

/// The corner at a tile's north-west: `+` if any of the four edges meeting there isn't open.
fn corner(grid: &Grid, pos: TilePos) -> char {
    let edges = [
        grid.north(pos.offset(-1, 0)),
        grid.north(pos),
        grid.west(pos.offset(0, -1)),
        grid.west(pos),
    ];
    if edges.iter().any(|&e| e != Edge::Open) {
        '+'
    } else {
        ' '
    }
}

/// Draws `w × h` tiles of the world from `origin` in the text grid format; `mark` can put a
/// character (an actor, say) over a cell.
pub fn render(
    grid: &Grid,
    origin: TilePos,
    w: i32,
    h: i32,
    mark: impl Fn(TilePos) -> Option<char>,
) -> String {
    let mut out = String::new();
    for cy in 0..=h {
        for cx in 0..=w {
            let pos = origin.offset(cx, cy);
            out.push(corner(grid, pos));
            if cx < w {
                out.push(edge_symbol(grid.north(pos), '-'));
            }
        }
        out.push('\n');
        if cy == h {
            break;
        }
        for cx in 0..=w {
            let pos = origin.offset(cx, cy);
            out.push(edge_symbol(grid.west(pos), '|'));
            if cx < w {
                out.push(mark(pos).unwrap_or_else(|| tile_symbol(grid.tile(pos))));
            }
        }
        out.push('\n');
    }
    out
}
