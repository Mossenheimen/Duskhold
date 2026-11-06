//! The world, and its tick.

use std::hash::{Hash, Hasher};

use crate::actor::{Actor, ActorId, ActorKind, Intent, Mind, Step};
use crate::arena::Arena;
use crate::geom::{Dir8, TilePos};
use crate::grid::Grid;
use crate::hash::{FastMap, Fnv64, Purpose, below, roll};
use crate::path::find_path;
use crate::senses::{Muffling, hears, line_of_sight};
use crate::text::{self, TextError, TextGrid, default_legend};

/// Ticks in a second of action time at 1×.
pub const TICKS_PER_SECOND: u32 = 10;

/// Thousandths of a tile in a straight step and in a diagonal one.
const STRAIGHT: u32 = 1000;
const DIAGONAL: u32 = 1414;

/// Tunable rules. (They move into the data files at build step 4.)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
    /// Creatures think every this many ticks, staggered by id.
    pub think_every: u64,
    /// Ticks a shambler keeps after something it heard or saw before giving up on it.
    pub forget_after: u64,
    /// How far footsteps carry, in tiles: walking, running.
    pub walk_noise: i32,
    pub run_noise: i32,
    pub muffling: Muffling,
    /// How far a shambler sees, in tiles. (Light comes at build step 5.)
    pub shambler_sight: i32,
    /// Out of 10,000: how often an idle shambler shuffles a step when it thinks.
    pub wander_chance: u32,
    /// Ticks waiting on a blocked path before planning it again.
    pub repath_after: u32,
    /// The most tiles one A* search explores, for the character and for the dead (who aren't
    /// clever: a way they can't find nearby, they don't find).
    pub path_budget: usize,
    pub dead_path_budget: usize,
}

impl Default for Rules {
    fn default() -> Rules {
        Rules {
            think_every: 5,
            forget_after: 300,
            walk_noise: 3,
            run_noise: 8,
            muffling: Muffling {
                wall: 4,
                shut_door: 2,
                shut_window: 1,
            },
            shambler_sight: 8,
            wander_chance: 1000,
            repath_after: 20,
            path_budget: 4096,
            dead_path_budget: 400,
        }
    }
}

/// Things that happened, for the client, logs and tests.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Event {
    /// A creature that wasn't after anything heard something at `at`.
    Heard { who: ActorId, at: TilePos },
    /// A creature that wasn't after anything saw `target`.
    Saw { who: ActorId, target: ActorId },
    /// A creature got next to `target`, close enough to step onto its tile.
    Reached { who: ActorId, target: ActorId },
    /// An actor asked to walk to `to` found no way there.
    NoWay { who: ActorId, to: TilePos },
}

/// A sound made this tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Sound {
    pub at: TilePos,
    /// How many tiles it carries in the open.
    pub loudness: i32,
    pub by: Option<ActorId>,
}

#[derive(Clone, Debug)]
pub struct World {
    pub seed: u64,
    pub tick: u64,
    pub rules: Rules,
    pub grid: Grid,
    actors: Arena<Actor>,
    characters: Vec<ActorId>,
    /// Who's on each tile, or stepping onto it.
    occupied: FastMap<TilePos, ActorId>,
    sounds: Vec<Sound>,
    events: Vec<Event>,
}

impl World {
    pub fn new(seed: u64) -> World {
        World {
            seed,
            tick: 0,
            rules: Rules::default(),
            grid: Grid::new(),
            actors: Arena::new(),
            characters: Vec::new(),
            occupied: FastMap::default(),
            sounds: Vec::new(),
            events: Vec::new(),
        }
    }

    /// A world drawn as a text grid (see [`text`]), its north-west cell at (0, 0) on the ground
    /// floor. `@` puts the character there, `z` a shambler. Returns the world and the actors
    /// placed, in reading order.
    pub fn from_text(seed: u64, map: &str) -> Result<(World, Vec<ActorId>), TextError> {
        let parsed = TextGrid::parse(map, default_legend)?;
        let mut world = World::new(seed);
        parsed.stamp(&mut world.grid, TilePos::new(0, 0, 0));
        let mut placed = Vec::new();
        for &(marker, x, y) in &parsed.markers {
            let kind = match marker {
                '@' => ActorKind::Character,
                'z' | 'Z' => ActorKind::Shambler,
                _ => continue,
            };
            placed.extend(world.spawn(kind, TilePos::new(x, y, 0)));
        }
        Ok((world, placed))
    }

    /// Puts an actor on a free, walkable tile.
    pub fn spawn(&mut self, kind: ActorKind, pos: TilePos) -> Option<ActorId> {
        if !self.grid.tile(pos).walkable() || self.occupied.contains_key(&pos) {
            return None;
        }
        let id = self.actors.insert(Actor::new(kind, pos));
        self.occupied.insert(pos, id);
        if kind == ActorKind::Character {
            self.characters.push(id);
        }
        Some(id)
    }

    pub fn actor(&self, id: ActorId) -> Option<&Actor> {
        self.actors.get(id)
    }

    /// Every actor, in id order.
    pub fn actors(&self) -> impl Iterator<Item = (ActorId, &Actor)> {
        self.actors.iter()
    }

    /// The first character placed: the player's, in single player.
    pub fn character(&self) -> Option<ActorId> {
        self.characters.first().copied()
    }

    /// Asks an actor to do something from now on.
    pub fn intent(&mut self, id: ActorId, intent: Intent) {
        if let Some(a) = self.actors.get_mut(id) {
            if a.intent != intent {
                a.path.clear();
                a.path_goal = None;
                a.blocked = 0;
            }
            a.intent = intent;
        }
    }

    /// Makes a sound at a tile, heard on the next tick (what a shout or a dropped pan does).
    pub fn make_sound(&mut self, at: TilePos, loudness: i32, by: Option<ActorId>) {
        self.sounds.push(Sound { at, loudness, by });
    }

    /// Advances the world by some ticks.
    pub fn step(&mut self, ticks: u32) {
        for _ in 0..ticks {
            self.tick_once();
        }
    }

    /// What happened since the last call.
    pub fn events(&mut self) -> std::vec::Drain<'_, Event> {
        self.events.drain(..)
    }

    /// A hash of everything that makes the game go on the way it does. The same seed and the same
    /// intents give the same hashes, tick for tick.
    pub fn state_hash(&self) -> u64 {
        let mut h = Fnv64::default();
        self.tick.hash(&mut h);
        self.grid.hash(&mut h);
        for (id, a) in self.actors.iter() {
            id.hash(&mut h);
            a.hash(&mut h);
        }
        h.finish()
    }

    /// Draws `w × h` tiles from `origin` as a text grid: `@` the character, `z` a shambler that
    /// isn't after anything, `Z` one that is.
    pub fn render(&self, origin: TilePos, w: i32, h: i32) -> String {
        let mut marks: FastMap<TilePos, char> = FastMap::default();
        for (_, a) in self.actors.iter() {
            let mark = match (a.kind, a.mind) {
                (ActorKind::Character, _) => '@',
                (
                    ActorKind::Shambler,
                    Mind::Shambler {
                        target: Some(_), ..
                    },
                ) => 'Z',
                (ActorKind::Shambler, _) => 'z',
            };
            marks.insert(a.pos, mark);
        }
        text::render(&self.grid, origin, w, h, |p| marks.get(&p).copied())
    }

    /// One tick, in a fixed order: movement, then hearing, then thinking (whose intents act on
    /// the next tick).
    fn tick_once(&mut self) {
        self.tick += 1;
        let ids = self.actors.ids();
        for &id in &ids {
            self.move_actor(id);
        }
        self.hear(&ids);
        for &id in &ids {
            self.think(id);
        }
        self.sounds.clear();
    }

    /// Starts a step if the actor's intent asks for one, then moves it along. The tile ahead is
    /// taken as the step starts, so two actors never share a tile.
    fn move_actor(&mut self, id: ActorId) {
        let rules = self.rules;
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        if a.step.is_none() {
            let (dir, run) = match next_heading(a, &self.grid, &rules) {
                Heading::Go(dir, run) => (dir, run),
                Heading::Stand => {
                    a.carry = 0;
                    return;
                }
                Heading::NoWay(to) => {
                    a.carry = 0;
                    self.events.push(Event::NoWay { who: id, to });
                    return;
                }
            };
            a.facing = dir;
            let to = a.pos.step(dir);
            if !self.grid.can_step(a.pos, dir) || self.occupied.contains_key(&to) {
                a.blocked += 1;
                a.carry = 0;
                return;
            }
            a.blocked = 0;
            if matches!(a.intent, Intent::WalkTo { .. }) {
                a.path.pop();
            }
            let length = if dir.is_diagonal() {
                DIAGONAL
            } else {
                STRAIGHT
            };
            a.step = Some(Step {
                from: a.pos,
                to,
                dir,
                progress: a.carry,
                length,
                run,
            });
            a.carry = 0;
            self.occupied.insert(to, id);
        }
        let Some(mut step) = a.step else { return };
        let speeds = a.kind.speeds();
        step.progress += (if step.run { speeds.run } else { speeds.walk }) / TICKS_PER_SECOND;
        if step.progress < step.length {
            a.step = Some(step);
            return;
        }
        a.step = None;
        a.pos = step.to;
        a.carry = step.progress - step.length;
        let footsteps = (a.kind == ActorKind::Character).then_some(if step.run {
            rules.run_noise
        } else {
            rules.walk_noise
        });
        self.occupied.remove(&step.from);
        if let Some(loudness) = footsteps {
            self.sounds.push(Sound {
                at: step.to,
                loudness,
                by: Some(id),
            });
        }
    }

    /// Creatures hear this tick's sounds: each goes after the nearest one it heard.
    fn hear(&mut self, ids: &[ActorId]) {
        if self.sounds.is_empty() {
            return;
        }
        let (rules, tick) = (self.rules, self.tick);
        for &id in ids {
            let Some(a) = self.actors.get(id) else {
                continue;
            };
            let Mind::Shambler {
                target, touching, ..
            } = a.mind
            else {
                continue;
            };
            let pos = a.pos;
            let mut nearest: Option<(i64, TilePos)> = None;
            for s in &self.sounds {
                if s.by != Some(id) && hears(&self.grid, s.at, pos, s.loudness, rules.muffling) {
                    let d2 = s.at.dist2(pos);
                    if nearest.is_none_or(|(best, _)| d2 < best) {
                        nearest = Some((d2, s.at));
                    }
                }
            }
            let Some((_, at)) = nearest else { continue };
            if target.is_none() {
                self.events.push(Event::Heard { who: id, at });
            }
            if let Some(a) = self.actors.get_mut(id) {
                a.mind = Mind::Shambler {
                    target: Some(at),
                    since: tick,
                    touching,
                };
            }
        }
    }

    /// A shambler's turn to think: look for prey, give up on old leads, decide where to go.
    fn think(&mut self, id: ActorId) {
        let (rules, tick, seed) = (self.rules, self.tick, self.seed);
        if !(tick + id.index() as u64).is_multiple_of(rules.think_every.max(1)) {
            return;
        }
        let Some(a) = self.actors.get(id) else { return };
        let Mind::Shambler {
            mut target,
            mut since,
            touching,
        } = a.mind
        else {
            return;
        };
        let pos = a.pos;

        let seen = self.nearest_visible_character(pos, rules.shambler_sight);
        if let Some((prey, at)) = seen {
            if target.is_none() {
                self.events.push(Event::Saw {
                    who: id,
                    target: prey,
                });
            }
            target = Some(at);
            since = tick;
        }
        if tick.saturating_sub(since) > rules.forget_after {
            target = None;
        }
        let next_to = seen
            .filter(|&(_, at)| self.can_reach(pos, at))
            .map(|(prey, _)| prey);
        if let Some(prey) = next_to
            && !touching
        {
            self.events.push(Event::Reached {
                who: id,
                target: prey,
            });
        }
        let intent = match (next_to, target) {
            // Next to its prey: it stops there. (Grabs and bites come with combat, build step 5.)
            (Some(_), _) => Intent::Idle,
            // Got to where the noise was, and nothing's there.
            (None, Some(goal)) if goal == pos => {
                target = None;
                Intent::Idle
            }
            (None, Some(goal)) => self.chase(a, goal),
            (None, None) => {
                let who = id.index() as u64;
                if below(roll(seed, tick, who, Purpose::Wander), 10_000) < rules.wander_chance {
                    let dir =
                        Dir8::ALL[below(roll(seed, tick, who, Purpose::WanderDir), 8) as usize];
                    Intent::Move { dir, run: false }
                } else {
                    Intent::Idle
                }
            }
        };
        if let Some(a) = self.actors.get_mut(id) {
            a.mind = Mind::Shambler {
                target,
                since,
                touching: next_to.is_some(),
            };
            a.intent = intent;
        }
    }

    /// How a shambler goes after `goal`: straight at it while it's in sight and the walls allow,
    /// otherwise along a path, through doors and round walls. Where it found no way, it presses
    /// toward the goal anyway, as the dead do against a wall.
    fn chase(&self, a: &Actor, goal: TilePos) -> Intent {
        let pos = a.pos;
        if line_of_sight(&self.grid, pos, goal)
            && let Some(dir) = self.open_step_toward(pos, goal)
        {
            return Intent::Move { dir, run: true };
        }
        if a.no_way.is_some_and(|tried| tried.steps_to(goal) <= 1) {
            return Intent::Move {
                dir: self.lurch_toward(pos, goal),
                run: true,
            };
        }
        // Keep the path already planned while the goal has only shifted by a tile.
        let to = match a.intent {
            Intent::WalkTo { to, .. } if to.steps_to(goal) <= 1 => to,
            _ => goal,
        };
        Intent::WalkTo { to, run: true }
    }

    /// The step straight at `goal`, when the walls allow it; None when they don't (then the way
    /// round needs a path: a step to one side would only look closer). Someone standing in that
    /// tile is gone round by an eighth of a turn if that's free, else waited behind.
    fn open_step_toward(&self, from: TilePos, goal: TilePos) -> Option<Dir8> {
        let dir = Dir8::nearest(goal.x - from.x, goal.y - from.y)?;
        if !self.grid.can_step(from, dir) {
            return None;
        }
        if !self.occupied.contains_key(&from.step(dir)) {
            return Some(dir);
        }
        for d in [dir.turned(1), dir.turned(-1)] {
            if self.grid.can_step(from, d) && !self.occupied.contains_key(&from.step(d)) {
                return Some(d);
            }
        }
        Some(dir)
    }

    /// Could an actor at `from` step onto `to`'s tile: next to it, with no wall or corner in the
    /// way? (Whoever stands there doesn't count.)
    fn can_reach(&self, from: TilePos, to: TilePos) -> bool {
        from.steps_to(to) == 1
            && Dir8::toward(to.x - from.x, to.y - from.y)
                .is_some_and(|d| self.grid.can_step(from, d))
    }

    /// The heading a shambler lurches in to reach `goal`: straight at it, or an eighth of a turn
    /// to either side when that way is blocked (by a wall or by another actor).
    fn lurch_toward(&self, from: TilePos, goal: TilePos) -> Dir8 {
        let Some(dir) = Dir8::nearest(goal.x - from.x, goal.y - from.y) else {
            return Dir8::S;
        };
        for d in [dir, dir.turned(1), dir.turned(-1)] {
            if self.grid.can_step(from, d) && !self.occupied.contains_key(&from.step(d)) {
                return d;
            }
        }
        dir
    }

    /// The nearest character within `range` tiles that `from` can see.
    fn nearest_visible_character(&self, from: TilePos, range: i32) -> Option<(ActorId, TilePos)> {
        if range <= 0 {
            return None;
        }
        let r2 = i64::from(range).pow(2);
        let mut best: Option<(i64, ActorId, TilePos)> = None;
        for &id in &self.characters {
            let Some(c) = self.actors.get(id) else {
                continue;
            };
            let d2 = from.dist2(c.pos);
            if d2 > r2 || best.is_some_and(|(b, ..)| d2 >= b) {
                continue;
            }
            if line_of_sight(&self.grid, from, c.pos) {
                best = Some((d2, id, c.pos));
            }
        }
        best.map(|(_, id, at)| (id, at))
    }
}

/// What an actor's intent asks of its legs right now.
enum Heading {
    Stand,
    Go(Dir8, bool),
    /// A `WalkTo` that found no way there.
    NoWay(TilePos),
}

/// The heading an actor's intent asks for now. Plans (and re-plans) the path of a `WalkTo`: when
/// the goal changed, when the next tile isn't next to it any more, or after waiting too long.
fn next_heading(a: &mut Actor, grid: &Grid, rules: &Rules) -> Heading {
    match a.intent {
        Intent::Idle => Heading::Stand,
        Intent::Move { dir, run } => Heading::Go(dir, run),
        Intent::WalkTo { to, run } => {
            if a.pos == to {
                a.intent = Intent::Idle;
                a.path.clear();
                a.path_goal = None;
                return Heading::Stand;
            }
            let stale = a.path_goal != Some(to)
                || a.path.last().is_none_or(|&next| a.pos.steps_to(next) != 1);
            if stale || a.blocked >= rules.repath_after {
                a.blocked = 0;
                let budget = match a.kind {
                    ActorKind::Shambler => rules.dead_path_budget,
                    ActorKind::Character => rules.path_budget,
                };
                match find_path(grid, a.pos, to, budget) {
                    Some(mut path) => {
                        path.reverse();
                        a.path = path;
                        a.path_goal = Some(to);
                        a.no_way = None;
                    }
                    None => {
                        a.intent = Intent::Idle;
                        a.path.clear();
                        a.path_goal = None;
                        a.no_way = Some(to);
                        return Heading::NoWay(to);
                    }
                }
            }
            match a
                .path
                .last()
                .and_then(|&next| Dir8::toward(next.x - a.pos.x, next.y - a.pos.y))
            {
                Some(dir) => Heading::Go(dir, run),
                None => Heading::Stand,
            }
        }
    }
}
