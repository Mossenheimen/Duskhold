//! Saves: a patch saved and loaded is the same world, and goes on the same way; a save holds
//! only what changed; and what isn't a save of this world is refused.

use dusk_core::save::{SaveError, peek};
use dusk_core::{Dir8, Edge, Intent, Item, Part, Skill, TilePos, World, WoundKind};
use dusk_data::{Data, default_dir, load};
use dusk_gen::patch;

fn data() -> Data {
    load(&default_dir()).unwrap_or_else(|e| panic!("{e}"))
}

/// Seed 1's longhouse door (`dusk-cli gen --seed 1` lists it), on the north edge of its tile.
const DOOR: TilePos = TilePos::new(100, 115, 0);

/// A patch that has been played: a walk, a door shut, a board's worth of damage on it, a wound,
/// something learnt, and a thing left on the ground.
fn played(data: &Data) -> World {
    let region = data.region("hushlands").unwrap();
    let (mut w, _) = patch(1, data, region);
    let me = w.character().unwrap();
    let door = (0..4)
        .map(|k| [Dir8::N, Dir8::S, Dir8::W, Dir8::E][k])
        .find(|&d| matches!(w.grid.edge(DOOR, d), Edge::Door { .. }))
        .expect("the longhouse's door is by its door tile");
    w.grid.set_edge(DOOR, door, Edge::door(false));
    w.put(TilePos::new(60, 130, 0), Item::new(0, 2));
    w.wound(me, WoundKind::Laceration, Part::LeftArm);
    w.give_xp(me, Skill::Athletics, 12_345);
    w.intent(
        me,
        Intent::WalkTo {
            to: TilePos::new(70, 128, 0),
            run: true,
        },
    );
    w.step(400);
    w
}

#[test]
fn a_saved_patch_loads_as_the_same_world_and_goes_on_the_same() {
    let data = data();
    let region = data.region("hushlands").unwrap();
    let mut a = played(&data);
    let bytes = dusk_gen::save(&a, region, b"what I remember");

    let header = peek(&bytes).unwrap();
    assert_eq!((header.seed, header.tick), (1, 400));
    assert_eq!(header.label, "hushlands");

    let loaded = dusk_gen::load(&data, &bytes).unwrap_or_else(|e| panic!("{e}"));
    let mut b = loaded.world;
    assert_eq!(loaded.memory, b"what I remember");
    assert_eq!(a.state_hash(), b.state_hash(), "the same world");
    assert_eq!(a.clock(), b.clock());

    // And it plays on the same: the same intents, the same hashes.
    let me = a.character().unwrap();
    assert_eq!(b.character(), Some(me));
    let back = Intent::WalkTo {
        to: TilePos::new(56, 130, 0),
        run: false,
    };
    a.intent(me, back);
    b.intent(me, back);
    for step in 0..20 {
        a.step(50);
        b.step(50);
        assert_eq!(
            a.state_hash(),
            b.state_hash(),
            "after {} ticks",
            (step + 1) * 50
        );
    }
    // A second save, of the loaded world, is the first world's save.
    assert_eq!(
        dusk_gen::save(&a, region, b""),
        dusk_gen::save(&b, region, b"")
    );
}

#[test]
fn a_save_holds_only_the_chunks_that_changed() {
    let data = data();
    let region = data.region("hushlands").unwrap();
    let (fresh, _) = patch(1, &data, region);
    assert!(
        fresh.grid.changed_chunks().is_empty(),
        "a new patch has changed nothing: the land comes back from the seed"
    );
    let w = played(&data);
    let changed: Vec<_> = w
        .grid
        .changed_chunks()
        .into_iter()
        .map(|(at, _)| at)
        .collect();
    assert_eq!(changed, vec![DOOR.chunk()], "the door's chunk, of 64");
    let bytes = dusk_gen::save(&w, region, b"");
    assert!(bytes.len() < 100_000, "{} bytes", bytes.len());
}

#[test]
fn what_isnt_a_save_of_this_world_is_refused() {
    let data = data();
    let region = data.region("hushlands").unwrap();
    let w = played(&data);
    let bytes = dusk_gen::save(&w, region, b"");

    assert_eq!(peek(b"not a save at all").unwrap_err(), SaveError::NotASave);
    assert_eq!(
        dusk_gen::load(&data, &bytes[..bytes.len() / 2]).err(),
        Some(SaveError::NotASave),
        "cut short"
    );
    // Onto another seed's world.
    let (mut other, _) = patch(2, &data, region);
    let before = other.state_hash();
    assert_eq!(other.restore(&bytes).unwrap_err(), SaveError::OtherWorld);
    assert_eq!(
        other.state_hash(),
        before,
        "and that world is left as it was"
    );
    // With other data: an item more than the save was made with.
    let (mut same, _) = patch(1, &data, region);
    let mut defs = (*same.defs).clone();
    let mut extra = defs.items[0].clone();
    extra.id = "something_new".into();
    defs.items.insert(0, extra);
    same.defs = std::sync::Arc::new(defs);
    assert_eq!(same.restore(&bytes).unwrap_err(), SaveError::DataChanged);
}

#[test]
fn a_save_keeps_the_pressure_and_what_became_of_each_body() {
    let data = data();
    let region = data.region("hushlands").unwrap();
    let (mut w, _) = patch(1, &data, region);
    assert_eq!(w.pressure(), 1000, "a patch starts at its region's start");
    assert_eq!(
        w.dead_about(),
        w.dead_wanted(),
        "with as many dead as it wants"
    );
    // Put one of the dead down, and let a day go by: time raises the pressure.
    let (z, _) = w
        .actors()
        .find(|(_, a)| !a.kind.is_character())
        .expect("the patch has its dead");
    for _ in 0..20 {
        w.wound(z, WoundKind::Bite, Part::Torso);
    }
    assert!(w.actor(z).unwrap().dead);
    w.step(36_000 / 2);
    let pressure = w.pressure();
    assert!(pressure > 1000, "half a day raised it");
    let bytes = dusk_gen::save(&w, region, &[]);
    let loaded = dusk_gen::load(&data, &bytes).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(loaded.world.pressure(), pressure);
    assert_eq!(
        loaded.world.actor(z).unwrap().remains,
        w.actor(z).unwrap().remains,
        "the body still gets up when it would have"
    );
    assert_eq!(loaded.world.state_hash(), w.state_hash());
}

#[test]
fn a_save_after_a_death_keeps_the_body_and_the_next_survivor() {
    let data = data();
    let region = data.region("hushlands").unwrap();
    let (mut w, _) = patch(2, &data, region);
    let first = w.character().unwrap();
    while !w.actor(first).unwrap().dead {
        w.wound(first, WoundKind::Bite, Part::Torso);
    }
    let next = w.next_character().expect("the next survivor");
    assert_ne!(w.name_of(first), None);
    assert!(w.is_hallowed(w.actor(next).unwrap().pos), "at the abbey");
    w.step(100);
    let bytes = dusk_gen::save(&w, region, &[]);
    let loaded = dusk_gen::load(&data, &bytes).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(loaded.world.character(), Some(next));
    assert_eq!(loaded.world.name_of(first), w.name_of(first));
    assert!(loaded.world.actor(first).unwrap().dead);
    assert_eq!(loaded.world.state_hash(), w.state_hash());
}
