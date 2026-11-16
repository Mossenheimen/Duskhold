//! How things look (`data/looks.toml`; `VISUALS.md` §5, the kit, and §14): the world's materials,
//! and every fixture as parts in four shapes, so they all look made by one hand. Nothing in the
//! core reads it: the client draws from it, through the bridge, and the tools picture it.
//!
//! A part is a unit shape (−½ to ½ each way) stretched to its `size`, turned as Godot turns a
//! node (y, then x, then z: `Basis::from_euler`), and set with its centre `at` a point of the
//! look, whose origin is the middle of its base, +y up, its front toward −z. The client makes the
//! same corners (`client/looks.gd`), so a silhouette worked out here is the one on screen.

use std::collections::BTreeMap;
use std::f32::consts::PI;
use std::path::Path;

use dusk_core::Ground;
use serde::Deserialize;

use crate::{DataError, Fixture, check_unique, fail, ground_named};

/// What a part is made in: a block, an upright cylinder (a cone, with no top), a ball, or a ridge
/// (a triangle `size.x` wide and `size.y` high, run `size.z` along).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Shape {
    Box,
    Cylinder,
    Ball,
    Prism,
}

impl Shape {
    /// Its number for the client (`DuskWorld.looks()`; `codes.gd`'s `Shape`).
    pub const fn code(self) -> u8 {
        match self {
            Shape::Box => 0,
            Shape::Cylinder => 1,
            Shape::Ball => 2,
            Shape::Prism => 3,
        }
    }
}

/// How a material's surface is drawn (`VISUALS.md` §6), by its code in the shaders.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Surface {
    #[default]
    Plain,
    Plaster,
    Wood,
    Stone,
    Foliage,
    Straw,
    Cloth,
    Iron,
    Bark,
}

impl Surface {
    /// Its code in the shaders (`surfaces.gdshaderinc`'s `sf_surface`; `codes.gd`'s `Surface`).
    pub const fn code(self) -> u8 {
        self as u8
    }
}

/// How a fixture stands on its tile.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Placed {
    /// Its back to a wall, a window or a fence of its own tile, if one has any; else as `Fixed`.
    #[default]
    Wall,
    /// Its front toward the camera's side: south.
    Fixed,
    /// Turned by a quarter at a time and a little more, by its place: crates, sacks, trees.
    Loose,
}

impl Placed {
    /// Its number for the client (`DuskWorld.looks()`; `codes.gd`'s `Placed`).
    pub const fn code(self) -> u8 {
        self as u8
    }
}

/// A colour (sRGB, 0 to 1) and what its surface is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Material {
    pub colour: [f32; 3],
    pub surface: Surface,
}

/// One part of a look, its material resolved.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Part {
    pub shape: Shape,
    /// Round a cylinder or a ball (3 to 16).
    pub sides: u8,
    /// A cylinder's top over its bottom (1 straight, 0 a cone).
    pub taper: f32,
    pub size: [f32; 3],
    pub at: [f32; 3],
    /// Degrees about x, y and z, as Godot's `rotation_degrees`.
    pub turn: [f32; 3],
    pub colour: [f32; 3],
    pub surface: Surface,
    /// Cut away round the character when it stands between it and the camera (a crown, a
    /// wardrobe); false for what you must see whole (a trunk).
    pub cut: bool,
}

/// How many numbers a part is in `Part::floats`.
pub const PART_FLOATS: usize = 17;

impl Part {
    /// Its turn as a matrix, rows by columns: Ry·Rx·Rz, as Godot's `Basis.from_euler`.
    pub fn basis(&self) -> [[f32; 3]; 3] {
        let [x, y, z] = self.turn.map(f32::to_radians);
        let (sx, cx) = x.sin_cos();
        let (sy, cy) = y.sin_cos();
        let (sz, cz) = z.sin_cos();
        let rx = [[1.0, 0.0, 0.0], [0.0, cx, -sx], [0.0, sx, cx]];
        let ry = [[cy, 0.0, sy], [0.0, 1.0, 0.0], [-sy, 0.0, cy]];
        let rz = [[cz, -sz, 0.0], [sz, cz, 0.0], [0.0, 0.0, 1.0]];
        mul(mul(ry, rx), rz)
    }

    /// Where a point of its unit shape lands in the look.
    pub fn place(&self, p: [f32; 3]) -> [f32; 3] {
        let b = self.basis();
        let s = [
            p[0] * self.size[0],
            p[1] * self.size[1],
            p[2] * self.size[2],
        ];
        let mut out = self.at;
        for (row, o) in b.iter().zip(out.iter_mut()) {
            *o += row[0] * s[0] + row[1] * s[1] + row[2] * s[2];
        }
        out
    }

    /// Its corners in the look: the points its silhouette is the hull of.
    pub fn points(&self) -> Vec<[f32; 3]> {
        unit_points(self.shape, self.sides, self.taper)
            .into_iter()
            .map(|p| self.place(p))
            .collect()
    }

    /// How many triangles it's drawn with (`client/looks.gd` makes the same).
    pub fn triangles(&self) -> usize {
        let n = usize::from(self.sides);
        match self.shape {
            Shape::Box => 12,
            Shape::Prism => 8,
            Shape::Cylinder if self.taper == 0.0 => n + (n - 2),
            Shape::Cylinder => 2 * n + 2 * (n - 2),
            Shape::Ball => n * (2 * rings(self.sides) - 2),
        }
    }

    /// The numbers the client builds it from: shape, sides, taper, size, at, turn (degrees),
    /// colour (sRGB), surface, cut.
    pub fn floats(&self) -> [f32; PART_FLOATS] {
        let [sx, sy, sz] = self.size;
        let [x, y, z] = self.at;
        let [tx, ty, tz] = self.turn;
        let [r, g, b] = self.colour;
        [
            f32::from(self.shape.code()),
            f32::from(self.sides),
            self.taper,
            sx,
            sy,
            sz,
            x,
            y,
            z,
            tx,
            ty,
            tz,
            r,
            g,
            b,
            f32::from(self.surface.code()),
            if self.cut { 1.0 } else { 0.0 },
        ]
    }
}

/// A ball's rings from pole to pole: half its sides, two at least.
pub fn rings(sides: u8) -> usize {
    usize::from(sides / 2).max(2)
}

/// A unit shape's corners (−½ to ½ each way), as the client makes them. Round shapes start half a
/// side round from +x, so an even-sided one has a flat side to each axis.
pub fn unit_points(shape: Shape, sides: u8, taper: f32) -> Vec<[f32; 3]> {
    let n = usize::from(sides);
    let round = |k: usize| {
        let a = PI / n as f32 + 2.0 * PI * k as f32 / n as f32;
        (0.5 * a.cos(), 0.5 * a.sin())
    };
    match shape {
        Shape::Box => {
            let mut out = Vec::with_capacity(8);
            for x in [-0.5, 0.5] {
                for y in [-0.5, 0.5] {
                    for z in [-0.5, 0.5] {
                        out.push([x, y, z]);
                    }
                }
            }
            out
        }
        Shape::Prism => vec![
            [-0.5, -0.5, -0.5],
            [0.5, -0.5, -0.5],
            [0.0, 0.5, -0.5],
            [-0.5, -0.5, 0.5],
            [0.5, -0.5, 0.5],
            [0.0, 0.5, 0.5],
        ],
        Shape::Cylinder => {
            let mut out = Vec::with_capacity(2 * n);
            for k in 0..n {
                let (x, z) = round(k);
                out.push([x, -0.5, z]);
            }
            if taper == 0.0 {
                out.push([0.0, 0.5, 0.0]);
            } else {
                for k in 0..n {
                    let (x, z) = round(k);
                    out.push([x * taper, 0.5, z * taper]);
                }
            }
            out
        }
        Shape::Ball => {
            let r = rings(sides);
            let mut out = vec![[0.0, 0.5, 0.0], [0.0, -0.5, 0.0]];
            for i in 1..r {
                let polar = PI * i as f32 / r as f32;
                let (s, c) = polar.sin_cos();
                for k in 0..n {
                    let (x, z) = round(k);
                    out.push([x * s, 0.5 * c, z * s]);
                }
            }
            out
        }
    }
}

fn mul(a: [[f32; 3]; 3], b: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
    let mut out = [[0.0; 3]; 3];
    for (i, row) in out.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            *cell = (0..3).map(|k| a[i][k] * b[k][j]).sum();
        }
    }
    out
}

/// A fixture's look.
#[derive(Clone, Debug, PartialEq)]
pub struct Look {
    /// One set of parts or more: a tile takes one by its place.
    pub variants: Vec<Vec<Part>>,
    pub placed: Placed,
    /// Its front turns toward the nearest of these fixtures near it (a chair to its table), or
    /// its back (a counter to the sister behind it), before `placed` says.
    pub faces: Option<String>,
    pub backs: Option<String>,
    /// How far, in tiles, it looks for what it faces or backs (1: only beside it).
    pub reach: u8,
    /// The least and most it's scaled by, one tile to the next.
    pub grow: [f32; 2],
    /// How far one tile's colour strays from the next's, lighter or darker (0 to 0.3).
    pub vary: f32,
    /// Where a fire laid in it burns (a hearth, a stove, a forge): its embers, from the origin.
    pub fire: Option<[f32; 3]>,
}

impl Look {
    /// How tall its first variant stands, in metres.
    pub fn height(&self) -> f32 {
        self.variants
            .first()
            .into_iter()
            .flatten()
            .flat_map(Part::points)
            .map(|p| p[1])
            .fold(0.0, f32::max)
    }

    /// The most triangles any of its variants is drawn with.
    pub fn triangles(&self) -> usize {
        let each = self
            .variants
            .iter()
            .map(|v| v.iter().map(Part::triangles).sum());
        each.max().unwrap_or(0)
    }
}

/// Everything in `looks.toml`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Looks {
    /// By "group.name": "ground.grass", "wood.oak".
    pub materials: BTreeMap<String, Material>,
    /// By fixture id.
    pub fixtures: BTreeMap<String, Look>,
    /// What's strewn on the ground, in the file's order.
    pub scatter: Vec<Scatter>,
}

/// Small things the client strews on the ground (a tuft of grass, a stone, reeds in the
/// shallows): only a look, on tiles with no fixture, never anything that stops you or can be
/// taken. Where each lies is worked out from its tile, the same every time.
#[derive(Clone, Debug, PartialEq)]
pub struct Scatter {
    pub id: String,
    /// The grounds it lies on.
    pub on: Vec<Ground>,
    /// The chance a tile of those grounds, with no fixture on it, has any of it.
    pub chance: f32,
    /// How many on a tile that has it: the fewest and the most.
    pub count: [u8; 2],
    /// Its parts, variants, size and tint (as a loose fixture's, turned any way).
    pub look: Look,
}

/// The materials the client reads by name (`look.gd`'s `take_materials`): the ground's kinds,
/// the buildings', and a bundle on a floor.
pub const NAMED: &[&str] = &[
    "ground.void",
    "ground.grass",
    "ground.dirt",
    "ground.road",
    "ground.water",
    "ground.floor",
    "ground.shallows",
    "ground.bridge",
    "build.wall",
    "build.door",
    "build.board",
    "build.fence",
    "world.litter",
];

/// The most triangles a fixture's look may have (`VISUALS.md` §5).
pub const FIXTURE_TRIANGLES: usize = 500;
/// The most a scatter's may: there are thousands of them.
pub const SCATTER_TRIANGLES: usize = 60;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LooksFile {
    #[serde(default)]
    material: BTreeMap<String, BTreeMap<String, MaterialFile>>,
    #[serde(default)]
    fixture: Vec<LookFile>,
    #[serde(default)]
    scatter: Vec<ScatterFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScatterFile {
    id: String,
    on: Vec<String>,
    chance: f32,
    count: [u8; 2],
    #[serde(default)]
    grow: Option<[f32; 2]>,
    #[serde(default)]
    vary: Option<f32>,
    #[serde(default)]
    parts: Vec<PartFile>,
    #[serde(default)]
    variant: Vec<VariantFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MaterialFile {
    colour: String,
    #[serde(default)]
    surface: Surface,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LookFile {
    id: String,
    #[serde(default)]
    placed: Placed,
    #[serde(default)]
    faces: Option<String>,
    #[serde(default)]
    backs: Option<String>,
    #[serde(default)]
    reach: Option<u8>,
    #[serde(default)]
    grow: Option<[f32; 2]>,
    #[serde(default)]
    vary: Option<f32>,
    #[serde(default)]
    fire: Option<[f32; 3]>,
    #[serde(default)]
    parts: Vec<PartFile>,
    #[serde(default)]
    variant: Vec<VariantFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VariantFile {
    parts: Vec<PartFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PartFile {
    shape: Shape,
    size: [f32; 3],
    #[serde(default)]
    at: [f32; 3],
    #[serde(default)]
    turn: [f32; 3],
    material: String,
    #[serde(default)]
    sides: Option<u8>,
    #[serde(default)]
    taper: Option<f32>,
    #[serde(default)]
    mirror: Option<Mirror>,
    #[serde(default)]
    repeat: Option<RepeatFile>,
    #[serde(default)]
    cut: Option<bool>,
}

/// A part's mirror image across x = 0, z = 0, or both (four of it).
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Mirror {
    X,
    Z,
    Xz,
}

/// `count` of a part, each `step` on from the last.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RepeatFile {
    count: u8,
    step: [f32; 3],
}

/// `looks.toml` (left out: no looks, and the client draws a plain box for each fixture). When
/// it's there, every fixture has a look, and the materials the client names are in it.
pub(crate) fn load(dir: &Path, fixtures: &[Fixture]) -> Result<Looks, DataError> {
    let file = dir.join("looks.toml");
    if !file.exists() {
        return Ok(Looks::default());
    }
    let text = std::fs::read_to_string(&file).map_err(|e| fail(&file, e.to_string()))?;
    let looks = parse(&file, &text, fixtures)?;
    if let Some(name) = NAMED.iter().find(|n| !looks.materials.contains_key(**n)) {
        return Err(fail(
            &file,
            format!("material {name:?} is missing: the client draws with it"),
        ));
    }
    if let Some(f) = fixtures
        .iter()
        .find(|f| !looks.fixtures.contains_key(&f.id))
    {
        return Err(fail(
            &file,
            format!("fixture {:?} has no look: every fixture needs one", f.id),
        ));
    }
    Ok(looks)
}

/// Reads looks from `text` (as if it were `file`), checking each against `fixtures`, but not
/// that every fixture has one.
pub fn parse(file: &Path, text: &str, fixtures: &[Fixture]) -> Result<Looks, DataError> {
    let raw: LooksFile = toml::from_str(text).map_err(|e| fail(file, e.to_string()))?;
    let mut materials = BTreeMap::new();
    for (group, names) in &raw.material {
        for (name, m) in names {
            let key = format!("{group}.{name}");
            let colour = hex(&m.colour).ok_or_else(|| {
                fail(
                    file,
                    format!(
                        "material {key:?}: colour is \"#RRGGBB\", not {:?}",
                        m.colour
                    ),
                )
            })?;
            materials.insert(
                key,
                Material {
                    colour,
                    surface: m.surface,
                },
            );
        }
    }
    check_unique(file, "fixture look", raw.fixture.iter().map(|l| &l.id))?;
    let mut looks = BTreeMap::new();
    for l in &raw.fixture {
        let bad = |m: String| fail(file, format!("fixture {:?}: {m}", l.id));
        let Some(fixture) = fixtures.iter().find(|f| f.id == l.id) else {
            return Err(bad("no such fixture in fixtures.toml".into()));
        };
        let look = to_look(l, &materials, fixtures).map_err(bad)?;
        match (fixture.fire > 0, look.fire.is_some()) {
            (true, false) => {
                return Err(bad(
                    "it takes a fire (fixtures.toml), so its look says where it burns: fire = \
                     [x, y, z]"
                        .into(),
                ));
            }
            (false, true) => return Err(bad("it takes no fire, so its look has none".into())),
            _ => {}
        }
        if look.triangles() > FIXTURE_TRIANGLES {
            return Err(bad(format!(
                "{} triangles, past the {FIXTURE_TRIANGLES} a fixture may have",
                look.triangles()
            )));
        }
        looks.insert(l.id.clone(), look);
    }
    check_unique(file, "scatter", raw.scatter.iter().map(|s| &s.id))?;
    let mut scatter = Vec::new();
    for s in &raw.scatter {
        let look = to_scatter(s, &materials);
        scatter.push(look.map_err(|m| fail(file, format!("scatter {:?}: {m}", s.id)))?);
    }
    Ok(Looks {
        materials,
        fixtures: looks,
        scatter,
    })
}

fn to_look(
    l: &LookFile,
    materials: &BTreeMap<String, Material>,
    fixtures: &[Fixture],
) -> Result<Look, String> {
    let variants = to_variants(&l.parts, &l.variant, materials)?;
    for (what, other) in [("faces", &l.faces), ("backs", &l.backs)] {
        if let Some(id) = other
            && !fixtures.iter().any(|f| &f.id == id)
        {
            return Err(format!("{what} {id:?}, which is no fixture"));
        }
    }
    let reach = l.reach.unwrap_or(1);
    if !(1..=12).contains(&reach) {
        return Err("reach is 1 to 12 tiles".into());
    }
    let (grow, vary) = to_grow(l.grow, l.vary)?;
    if let Some(at) = l.fire
        && at.iter().any(|x| !(-1.0..=3.0).contains(x))
    {
        return Err("fire is a point of the look: x, y, z within a few metres of it".into());
    }
    Ok(Look {
        variants,
        placed: l.placed,
        faces: l.faces.clone(),
        backs: l.backs.clone(),
        reach,
        grow,
        vary,
        fire: l.fire,
    })
}

/// A look's parts, or its variants' (one or the other), each with its mirror images and repeats.
fn to_variants(
    parts: &[PartFile],
    variant: &[VariantFile],
    materials: &BTreeMap<String, Material>,
) -> Result<Vec<Vec<Part>>, String> {
    let sets: Vec<&[PartFile]> = match (parts.is_empty(), variant.is_empty()) {
        (false, true) => vec![parts],
        (true, false) => variant.iter().map(|v| &v.parts[..]).collect(),
        (false, false) => return Err("parts, or [[variant]]s of parts: not both".into()),
        (true, true) => return Err("no parts".into()),
    };
    let mut variants = Vec::new();
    for (v, set) in sets.into_iter().enumerate() {
        if set.is_empty() {
            return Err(format!("variant {} has no parts", v + 1));
        }
        let mut out = Vec::new();
        for (n, p) in set.iter().enumerate() {
            to_parts(p, materials, &mut out).map_err(|m| format!("part {}: {m}", n + 1))?;
        }
        variants.push(out);
    }
    Ok(variants)
}

/// A look's `grow` and `vary`, checked, or the defaults (the same size; 4.5%).
fn to_grow(grow: Option<[f32; 2]>, vary: Option<f32>) -> Result<([f32; 2], f32), String> {
    let grow = grow.unwrap_or([1.0, 1.0]);
    if !(0.3..=3.0).contains(&grow[0]) || !(grow[0]..=3.0).contains(&grow[1]) {
        return Err("grow is [least, most], each 0.3 to 3, the least first".into());
    }
    let vary = vary.unwrap_or(0.045);
    if !(0.0..=0.3).contains(&vary) {
        return Err("vary is 0 to 0.3".into());
    }
    Ok((grow, vary))
}

fn to_scatter(s: &ScatterFile, materials: &BTreeMap<String, Material>) -> Result<Scatter, String> {
    let mut on = Vec::new();
    for name in &s.on {
        match ground_named(name) {
            Some(g) if g != Ground::Void => on.push(g),
            _ => return Err(format!("on: no ground called {name:?}")),
        }
    }
    if on.is_empty() {
        return Err("on names the grounds it lies on".into());
    }
    if !(s.chance > 0.0 && s.chance <= 1.0) {
        return Err("chance is above 0 and at most 1".into());
    }
    if !(1..=6).contains(&s.count[0]) || !(s.count[0]..=6).contains(&s.count[1]) {
        return Err("count is [fewest, most], each 1 to 6, the fewest first".into());
    }
    let (grow, vary) = to_grow(s.grow, s.vary)?;
    let look = Look {
        variants: to_variants(&s.parts, &s.variant, materials)?,
        placed: Placed::Loose,
        faces: None,
        backs: None,
        reach: 1,
        grow,
        vary,
        fire: None,
    };
    if look.triangles() > SCATTER_TRIANGLES {
        return Err(format!(
            "{} triangles, past the {SCATTER_TRIANGLES} a scatter may have",
            look.triangles()
        ));
    }
    Ok(Scatter {
        id: s.id.clone(),
        on,
        chance: s.chance,
        count: s.count,
        look,
    })
}

/// A part from the file, and its mirror images and repeats, into `out`.
fn to_parts(
    p: &PartFile,
    materials: &BTreeMap<String, Material>,
    out: &mut Vec<Part>,
) -> Result<(), String> {
    let m = materials
        .get(&p.material)
        .ok_or_else(|| format!("material {:?} isn't in [material]", p.material))?;
    if p.size.iter().any(|s| !(*s > 0.0 && *s <= 12.0)) {
        return Err("size is above 0 and at most 12 metres each way".into());
    }
    if p.at.iter().any(|a| !(-6.0..=6.0).contains(a)) {
        return Err("at is within 6 metres of the look's origin".into());
    }
    if p.turn.iter().any(|t| !(-360.0..=360.0).contains(t)) {
        return Err("turn is degrees, -360 to 360".into());
    }
    let round = matches!(p.shape, Shape::Cylinder | Shape::Ball);
    let sides = match (round, p.sides) {
        (true, Some(s)) if (3..=16).contains(&s) => s,
        (true, Some(_)) => return Err("sides is 3 to 16".into()),
        (true, None) => 8,
        (false, Some(_)) => return Err("only a cylinder or a ball has sides".into()),
        (false, None) => 4,
    };
    let taper = match (p.shape, p.taper) {
        (Shape::Cylinder, Some(t)) if (0.0..=3.0).contains(&t) => t,
        (Shape::Cylinder, Some(_)) => return Err("taper is 0 (a cone) to 3".into()),
        (_, Some(_)) => return Err("only a cylinder has a taper".into()),
        (_, None) => 1.0,
    };
    let part = Part {
        shape: p.shape,
        sides,
        taper,
        size: p.size,
        at: p.at,
        turn: p.turn,
        colour: m.colour,
        surface: m.surface,
        cut: p.cut.unwrap_or(true),
    };
    let mut copies = vec![part];
    if let Some(r) = &p.repeat {
        if !(2..=16).contains(&r.count) {
            return Err("repeat's count is 2 to 16".into());
        }
        for k in 1..r.count {
            let k = f32::from(k);
            let mut c = part;
            for (a, s) in c.at.iter_mut().zip(r.step) {
                *a += k * s;
            }
            copies.push(c);
        }
    }
    let flip_x = |mut c: Part| {
        c.at[0] = -c.at[0];
        c.turn[1] = -c.turn[1];
        c.turn[2] = -c.turn[2];
        c
    };
    let flip_z = |mut c: Part| {
        c.at[2] = -c.at[2];
        c.turn[0] = -c.turn[0];
        c.turn[1] = -c.turn[1];
        c
    };
    let mirrored: Vec<Part> = match p.mirror {
        None => Vec::new(),
        Some(Mirror::X) => copies.iter().map(|c| flip_x(*c)).collect(),
        Some(Mirror::Z) => copies.iter().map(|c| flip_z(*c)).collect(),
        Some(Mirror::Xz) => {
            let xs: Vec<Part> = copies.iter().map(|c| flip_x(*c)).collect();
            let mut all = xs.clone();
            all.extend(copies.iter().map(|c| flip_z(*c)));
            all.extend(xs.into_iter().map(flip_z));
            all
        }
    };
    out.extend(copies);
    out.extend(mirrored);
    Ok(())
}

/// "#RRGGBB" as sRGB, 0 to 1.
fn hex(text: &str) -> Option<[f32; 3]> {
    let digits = text.strip_prefix('#')?;
    if digits.len() != 6 || !digits.is_ascii() {
        return None;
    }
    let byte = |i: usize| u8::from_str_radix(&digits[i..i + 2], 16).ok();
    Some([byte(0)?, byte(2)?, byte(4)?].map(|b| f32::from(b) / 255.0))
}
