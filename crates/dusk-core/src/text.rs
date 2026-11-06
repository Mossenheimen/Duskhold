//! The text grid: how buildings are written (prefabs), how tests draw their maps, and how the
//! tools print the world. Rows alternate:
//!
//! - even rows are edges: `+` corners (or spaces) between horizontal edges;
//! - odd rows are cells: vertical edges between cells, so cells sit in the odd columns.
//!
//! On an edge, `-` (horizontal) or `|` (vertical) is a wall, `D` a shut door, `d` an open one,
//! `W` a shut window, `w` an open one, and a space is open. A cell's character comes from the
//! legend.
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

/// What one cell's character stands for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub tile: Tile,
    /// A letter or `@` on the cell, such as a spawn point.
    pub marker: Option<char>,
}

/// The default legend: `.` a floor (indoors), `,` earth (outdoors), `#` furniture or rock, a
/// space nothing at all. A letter or `@` marks a floor tile (tests and the demo use `@` for the
/// character and `z` for a shambler).
pub fn default_legend(ch: char) -> Option<Cell> {
    let tile = match ch {
        '.' => Tile::FLOOR,
        ',' => Tile::EARTH,
        '#' => Tile {
            solid: true,
            ..Tile::FLOOR
        },
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

/// A parsed text grid, ready to stamp into a world.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextGrid {
    pub width: i32,
    pub height: i32,
    /// `width × height`, row by row.
    cells: Vec<Tile>,
    /// `width × (height + 1)`: row `height` is the south side.
    north: Vec<Edge>,
    /// `(width + 1) × height`: column `width` is the east side.
    west: Vec<Edge>,
    /// Each marker with its cell, in reading order.
    pub markers: Vec<(char, i32, i32)>,
}

impl TextGrid {
    pub fn parse(text: &str, legend: impl Fn(char) -> Option<Cell>) -> Result<TextGrid, TextError> {
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

        let mut grid = TextGrid {
            width: width as i32,
            height: height as i32,
            cells: Vec::with_capacity(width * height),
            north: Vec::with_capacity(width * (height + 1)),
            west: Vec::with_capacity((width + 1) * height),
            markers: Vec::new(),
        };
        for cy in 0..height {
            for cx in 0..width {
                let (r, c) = (2 * cy + 1, 2 * cx + 1);
                let cell = legend(at(r, c)).ok_or_else(|| err(r, c, "a cell from the legend"))?;
                grid.cells.push(cell.tile);
                if let Some(marker) = cell.marker {
                    grid.markers.push((marker, cx as i32, cy as i32));
                }
            }
        }
        for cy in 0..=height {
            let r = 2 * cy;
            for cx in 0..width {
                let c = 2 * cx + 1;
                let edge = edge_from(at(r, c), '-')
                    .ok_or_else(|| err(r, c, "an edge: '-', 'D', 'd', 'W', 'w' or a space"))?;
                grid.north.push(edge);
            }
            for cx in 0..=width {
                if !matches!(at(r, 2 * cx), '+' | ' ') {
                    return Err(err(r, 2 * cx, "a corner: '+' or a space"));
                }
            }
        }
        for cy in 0..height {
            let r = 2 * cy + 1;
            for cx in 0..=width {
                let c = 2 * cx;
                let edge = edge_from(at(r, c), '|')
                    .ok_or_else(|| err(r, c, "an edge: '|', 'D', 'd', 'W', 'w' or a space"))?;
                grid.west.push(edge);
            }
        }
        Ok(grid)
    }

    /// Writes the grid into the world, its north-west cell at `origin`.
    pub fn stamp(&self, grid: &mut Grid, origin: TilePos) {
        let (w, h) = (self.width, self.height);
        for cy in 0..h {
            for cx in 0..w {
                grid.set_tile(origin.offset(cx, cy), self.cells[(cy * w + cx) as usize]);
            }
        }
        for cy in 0..=h {
            for cx in 0..w {
                grid.set_north(origin.offset(cx, cy), self.north[(cy * w + cx) as usize]);
            }
        }
        for cy in 0..h {
            for cx in 0..=w {
                grid.set_west(
                    origin.offset(cx, cy),
                    self.west[(cy * (w + 1) + cx) as usize],
                );
            }
        }
    }
}

fn edge_from(ch: char, wall: char) -> Option<Edge> {
    Some(match ch {
        ' ' => Edge::Open,
        'D' => Edge::Door { open: false },
        'd' => Edge::Door { open: true },
        'W' => Edge::Window { open: false },
        'w' => Edge::Window { open: true },
        c if c == wall => Edge::Wall,
        _ => return None,
    })
}

fn edge_symbol(edge: Edge, wall: char) -> char {
    match edge {
        Edge::Open => ' ',
        Edge::Wall => wall,
        Edge::Door { open: false } => 'D',
        Edge::Door { open: true } => 'd',
        Edge::Window { open: false } => 'W',
        Edge::Window { open: true } => 'w',
    }
}

fn cell_symbol(tile: Tile) -> char {
    if tile.solid {
        return '#';
    }
    match tile.ground {
        Ground::Void => ' ',
        Ground::Earth => ',',
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
                out.push(mark(pos).unwrap_or_else(|| cell_symbol(grid.tile(pos))));
            }
        }
        out.push('\n');
    }
    out
}
