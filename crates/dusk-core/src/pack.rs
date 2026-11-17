//! The pack as Path of Exile's (`CONCEPT.md`, the first playtest: "the inventory a grid of icons
//! round a figure"; the user, 2026-11-16: "PoE 2's grid as the rule: what fits is what you carry,
//! and you arrange it by hand").
//!
//! - **Grids of cells:** your pockets (`Defs::pockets`), the bag on your back (its `Bag::grid`)
//!   and each container (`Defs::grids`, by fixture) are so many cells across and down. Every
//!   item takes up so many (`Defs::size`: a bandage one, a sword one across and three down), and
//!   a stack sits with its top-left on a cell (`Item::cell`), its cells its own.
//! - **What fits is what you carry:** a stack goes where its cells are free, or onto a stack of
//!   the same with room, which takes no more cells. Weight no longer bounds what fits; it still
//!   slows you (`World::weighed_down`).
//! - **Where it goes:** where you put it, if it fits there; else the first free place down each
//!   column in turn, from the left (Path of Exile's).
//! - **Arranging is a choice, not an act:** moving a stack within its own grid takes no time
//!   (`World::arrange`, `World::arrange_store`). Moving it between grids is the act it always was
//!   (taking, putting away, between the pockets and the bag: `bag.rs`).
//! - **No room:** what you take is bounded by the room you have. What comes to you otherwise
//!   (made, bought, picked, coin, put away from your hand) and doesn't fit falls at your feet,
//!   and you're told (`Event::NoRoom`).
//! - **The floor** is no grid: anything can lie on it. A stack there has no cell.
//! - **A stack with no cell** in a grid takes up none: an older save's, or a tool's that gave
//!   more than fits. An older save's pack is laid out as it loads (`World::settle`).

use crate::actor::ActorId;
use crate::geom::TilePos;
use crate::items::{Defs, Item, within_reach};
use crate::world::World;

/// A cell of a grid: across, then down, from its top-left.
pub type Cell = (u8, u8);

/// What's taken in a grid of cells.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cells {
    w: u8,
    h: u8,
    taken: Vec<bool>,
}

impl Cells {
    /// An empty grid, `size` across and down.
    pub fn new(size: (u8, u8)) -> Cells {
        let (w, h) = size;
        Cells {
            w,
            h,
            taken: vec![false; usize::from(w) * usize::from(h)],
        }
    }

    /// How many cells across and down.
    pub fn size(&self) -> (u8, u8) {
        (self.w, self.h)
    }

    fn index(&self, x: u8, y: u8) -> usize {
        usize::from(y) * usize::from(self.w) + usize::from(x)
    }

    /// Whether a thing of `size` fits with its top-left on `at`: inside the grid, on free cells.
    pub fn fits(&self, at: Cell, size: (u8, u8)) -> bool {
        let ((x, y), (w, h)) = (at, size);
        if w == 0 || h == 0 {
            return false;
        }
        if u16::from(x) + u16::from(w) > u16::from(self.w)
            || u16::from(y) + u16::from(h) > u16::from(self.h)
        {
            return false;
        }
        (y..y + h).all(|cy| (x..x + w).all(|cx| !self.taken[self.index(cx, cy)]))
    }

    /// Marks a thing's cells as taken: what's inside the grid of them.
    pub fn take(&mut self, at: Cell, size: (u8, u8)) {
        let ((x, y), (w, h)) = (at, size);
        for cy in y..y.saturating_add(h).min(self.h) {
            for cx in x..x.saturating_add(w).min(self.w) {
                let i = self.index(cx, cy);
                self.taken[i] = true;
            }
        }
    }

    /// The first place a thing of `size` fits: down each column in turn, from the left.
    pub fn first(&self, size: (u8, u8)) -> Option<Cell> {
        (0..self.w)
            .flat_map(|x| (0..self.h).map(move |y| (x, y)))
            .find(|&at| self.fits(at, size))
    }

    /// How many cells are taken.
    pub fn used(&self) -> usize {
        self.taken.iter().filter(|&&t| t).count()
    }

    /// Whether `cell` is inside the grid.
    pub fn holds(&self, cell: Cell) -> bool {
        cell.0 < self.w && cell.1 < self.h
    }
}

/// Whether a stack sitting with its top-left on `at`, `size` big, covers `cell`.
pub fn covers(at: Cell, size: (u8, u8), cell: Cell) -> bool {
    cell.0 >= at.0 && cell.1 >= at.1 && cell.0 - at.0 < size.0 && cell.1 - at.1 < size.1
}

/// The stack of `list` in a grid (`here`) whose cells cover `cell`.
pub(crate) fn stack_at(
    defs: &Defs,
    list: &[Item],
    here: impl Fn(&Item) -> bool,
    cell: Cell,
) -> Option<usize> {
    list.iter()
        .position(|i| here(i) && i.cell.is_some_and(|at| covers(at, defs.size(i.def), cell)))
}

/// What's taken in a grid (`size`) by the stacks of `list` in it (`here`), but the one at `skip`.
pub(crate) fn cells_of(
    defs: &Defs,
    size: (u8, u8),
    list: &[Item],
    here: impl Fn(&Item) -> bool,
    skip: Option<usize>,
) -> Cells {
    let mut cells = Cells::new(size);
    for (k, i) in list.iter().enumerate() {
        if Some(k) != skip
            && here(i)
            && let Some(at) = i.cell
        {
            cells.take(at, defs.size(i.def));
        }
    }
    cells
}

/// Puts as many of `item` into the grid `cells` maps, in `list`, as there's room for (`here`
/// says which of the list's stacks are in that grid; `item` comes as it should be there, its
/// flags set). Asked for a place (`at`), onto the stack alike that covers it, or a new stack
/// there; then onto the stacks alike with room, then new stacks of up to a stack each, on the
/// first free cells. What's left over: none when it all fit.
pub(crate) fn place(
    defs: &Defs,
    list: &mut Vec<Item>,
    cells: &mut Cells,
    item: Item,
    here: impl Fn(&Item) -> bool,
    at: Option<Cell>,
) -> u16 {
    let size = defs.size(item.def);
    let stack = defs.item(item.def).map_or(1, |d| d.stack.max(1));
    let mut left = item.count;
    let new = |c: Cell, n: u16| Item {
        count: n,
        cell: Some(c),
        ..item
    };
    if let Some(at) = at.filter(|_| left > 0) {
        match stack_at(defs, list, &here, at) {
            Some(k) if defs.alike(&list[k], &item) && list[k].count < stack => {
                let n = (stack - list[k].count).min(left);
                list[k].count += n;
                left -= n;
            }
            Some(_) => {}
            None if cells.fits(at, size) => {
                let n = left.min(stack);
                cells.take(at, size);
                list.push(new(at, n));
                left -= n;
            }
            None => {}
        }
    }
    for held in list.iter_mut() {
        if left == 0 {
            break;
        }
        if here(held) && held.cell.is_some() && defs.alike(held, &item) && held.count < stack {
            let n = (stack - held.count).min(left);
            held.count += n;
            left -= n;
        }
    }
    while left > 0 {
        let Some(c) = cells.first(size) else {
            break;
        };
        let n = left.min(stack);
        cells.take(c, size);
        list.push(new(c, n));
        left -= n;
    }
    left
}

/// Moves (or merges) the stack at `slot` of `list` onto `to`, within the grid it's in (`here`,
/// `size` across and down): onto the stack alike that covers `to`, as many as it has room for,
/// or the whole stack there if its cells are free. How many moved; none if it can't.
pub(crate) fn shuffle(
    defs: &Defs,
    list: &mut Vec<Item>,
    size: (u8, u8),
    here: impl Fn(&Item) -> bool,
    slot: usize,
    to: Cell,
) -> u16 {
    let Some(&item) = list.get(slot).filter(|i| here(i)) else {
        return 0;
    };
    let stack = defs.item(item.def).map_or(1, |d| d.stack.max(1));
    let under = stack_at(defs, list, &here, to).filter(|&k| k != slot);
    if let Some(k) = under {
        if !defs.alike(&list[k], &item) || list[k].count >= stack {
            return 0;
        }
        let n = (stack - list[k].count).min(item.count);
        list[k].count += n;
        if n >= item.count {
            list.remove(slot);
        } else {
            list[slot].count -= n;
        }
        return n;
    }
    let cells = cells_of(defs, size, list, &here, Some(slot));
    if !cells.fits(to, defs.size(item.def)) {
        return 0;
    }
    list[slot].cell = Some(to);
    item.count
}

impl World {
    /// One of `id`'s grids, across and down: its pockets', or the bag's on its back (`bagged`;
    /// None with no bag on).
    pub fn pack_grid(&self, id: ActorId, bagged: bool) -> Option<(u8, u8)> {
        self.actors.get(id)?;
        if bagged {
            self.worn_bag(id).map(|(_, b)| b.grid)
        } else {
            Some(self.defs.pockets)
        }
    }

    /// What's taken in one of `id`'s grids (`bagged`: the bag's), but the stack at `skip`.
    pub fn pack_cells(&self, id: ActorId, bagged: bool, skip: Option<usize>) -> Option<Cells> {
        let size = self.pack_grid(id, bagged)?;
        let a = self.actors.get(id)?;
        Some(cells_of(&self.defs, size, &a.items, in_grid(bagged), skip))
    }

    /// The grid of the container on `at`, across and down; None for a floor.
    pub fn store_grid(&self, at: TilePos) -> Option<(u8, u8)> {
        let fixture = self.grid.tile(at).fixture;
        self.defs
            .container(fixture)
            .map(|_| self.defs.container_grid(fixture))
    }

    /// Puts as many of `item` into one of `id`'s grids as fit (`bagged`: the bag's), at `at` if
    /// it's free there (or onto the stack alike there). What's left over.
    pub(crate) fn stow_grid(
        &mut self,
        id: ActorId,
        item: Item,
        bagged: bool,
        at: Option<Cell>,
    ) -> u16 {
        let Some(mut cells) = self.pack_cells(id, bagged, None) else {
            return item.count;
        };
        let defs = self.defs.clone();
        let Some(a) = self.actors.get_mut(id) else {
            return item.count;
        };
        let piece = Item {
            bagged,
            belt: 0,
            cell: None,
            ..item
        };
        place(&defs, &mut a.items, &mut cells, piece, in_grid(bagged), at)
    }

    /// How many of the stack at `slot` of `id`'s pack (still `def`) would move onto `to` in its
    /// own grid: the whole stack to free cells, or onto a stack alike there as many as it has room
    /// for; 0 if it can't. Arranging takes no time.
    pub fn can_arrange(&self, id: ActorId, slot: u16, def: u16, to: Cell) -> u16 {
        let Some(a) = self.actors.get(id).filter(|a| !a.dead) else {
            return 0;
        };
        let Some(item) = a
            .items
            .get(usize::from(slot))
            .filter(|i| i.def == def && i.belt == 0)
        else {
            return 0;
        };
        let Some(size) = self.pack_grid(id, item.bagged) else {
            return 0;
        };
        let mut list = a.items.clone();
        shuffle(
            &self.defs,
            &mut list,
            size,
            in_grid(item.bagged),
            usize::from(slot),
            to,
        )
    }

    /// Moves the stack at `slot` of `id`'s pack (still `def`) onto `to` in its own grid (see
    /// `can_arrange`): at once, a choice and not an act. Whether it moved.
    pub fn arrange(&mut self, id: ActorId, slot: u16, def: u16, to: Cell) -> bool {
        if self.can_arrange(id, slot, def, to) == 0 {
            return false;
        }
        let defs = self.defs.clone();
        let Some(bagged) = self
            .actors
            .get(id)
            .and_then(|a| a.items.get(usize::from(slot)))
            .map(|i| i.bagged)
        else {
            return false;
        };
        let Some(size) = self.pack_grid(id, bagged) else {
            return false;
        };
        let Some(a) = self.actors.get_mut(id) else {
            return false;
        };
        shuffle(
            &defs,
            &mut a.items,
            size,
            in_grid(bagged),
            usize::from(slot),
            to,
        ) > 0
    }

    /// How many of the stack at `slot` on `at` (still `def`), a container within `id`'s reach,
    /// would move onto `to` in its grid (see `can_arrange`).
    pub fn can_arrange_store(
        &self,
        id: ActorId,
        at: TilePos,
        slot: u16,
        def: u16,
        to: Cell,
    ) -> u16 {
        let Some(a) = self.actors.get(id).filter(|a| !a.dead) else {
            return 0;
        };
        let Some(size) = self.store_grid(at) else {
            return 0;
        };
        if !within_reach(&self.grid, a.pos, at) {
            return 0;
        }
        let mut held = self.store(at);
        if held.get(usize::from(slot)).is_none_or(|i| i.def != def) {
            return 0;
        }
        shuffle(&self.defs, &mut held, size, |_| true, usize::from(slot), to)
    }

    /// Moves the stack at `slot` of the container on `at` (still `def`) onto `to` in its grid:
    /// at once. Whether it moved.
    pub fn arrange_store(
        &mut self,
        id: ActorId,
        at: TilePos,
        slot: u16,
        def: u16,
        to: Cell,
    ) -> bool {
        if self.can_arrange_store(id, at, slot, def, to) == 0 {
            return false;
        }
        let Some(size) = self.store_grid(at) else {
            return false;
        };
        let mut held = self.store(at);
        let moved = shuffle(&self.defs, &mut held, size, |_| true, usize::from(slot), to) > 0;
        self.stores.insert(at, held);
        moved
    }

    /// How many of `item` the container on `at` has room for, at `to` first if it's asked for.
    pub fn store_room(&self, at: TilePos, item: &Item, to: Option<Cell>) -> u16 {
        let Some(size) = self.store_grid(at) else {
            return item.count;
        };
        let mut held = self.store(at);
        let mut cells = cells_of(&self.defs, size, &held, |_| true, None);
        let piece = Item {
            cell: None,
            ..item.loose()
        };
        item.count - place(&self.defs, &mut held, &mut cells, piece, |_| true, to)
    }

    /// A container's contents laid out on its grid: each stack with no cell yet onto the first
    /// free one (what doesn't fit keeps none). What's on a floor is as it is.
    pub(crate) fn laid_out(&self, at: TilePos, mut held: Vec<Item>) -> Vec<Item> {
        let Some(size) = self.store_grid(at) else {
            return held;
        };
        let mut cells = cells_of(&self.defs, size, &held, |_| true, None);
        for i in held.iter_mut().filter(|i| i.cell.is_none()) {
            let s = self.defs.size(i.def);
            if let Some(c) = cells.first(s) {
                cells.take(c, s);
                i.cell = Some(c);
            }
        }
        held
    }

    /// Lays out what has no place yet (an older save's, as it loads): each character's pack, a
    /// stack into its own grid, else the other, else at its feet; and each container's.
    #[cfg(feature = "save")]
    pub(crate) fn settle(&mut self) {
        for i in 0..self.characters.len() {
            let id = self.characters[i];
            let Some(a) = self.actors.get_mut(id).filter(|a| !a.dead) else {
                continue;
            };
            let (unplaced, rest): (Vec<Item>, Vec<Item>) = std::mem::take(&mut a.items)
                .into_iter()
                .partition(|i| i.belt == 0 && i.cell.is_none());
            a.items = rest;
            for item in unplaced {
                let first = item.bagged && self.worn_bag(id).is_some();
                let left = self.stow_grid(id, item, first, None);
                let left = match left {
                    0 => 0,
                    n => self.stow_grid(id, Item { count: n, ..item }, !first, None),
                };
                if left > 0 {
                    self.fall_at_feet(
                        id,
                        Item {
                            count: left,
                            ..item
                        },
                    );
                }
            }
        }
        let containers: Vec<TilePos> = self
            .stores
            .keys()
            .copied()
            .filter(|&at| self.store_grid(at).is_some())
            .collect();
        for at in containers {
            let held = self.stores.remove(&at).unwrap_or_default();
            let laid = self.laid_out(at, held);
            self.stores.insert(at, laid);
        }
    }
}

/// Which of a pack's stacks are in a grid: off the belt, in the bag or not.
pub(crate) fn in_grid(bagged: bool) -> impl Fn(&Item) -> bool + Copy {
    move |i: &Item| i.belt == 0 && i.bagged == bagged
}
