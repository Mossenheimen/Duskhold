//! Saves: a patch saved and loaded is the same world, and goes on the same way; a save holds
//! only what changed; and what isn't a save of this world is refused.

use dusk_core::save::{SaveError, peek};
use dusk_core::{Dir8, Edge, Intent, Item, Part, Skill, TilePos, World, WoundKind};
use dusk_data::{Data, default_dir, load};
use dusk_gen::patch;

fn data() -> Data {
    load(&default_dir()).unwrap_or_else(|e| panic!("{e}"))
}

/// A patch that has been played: a walk, a door shut (the front door of the town's first
/// building, the chapel: this returns the tile just inside it too), a wound, something learnt,
/// and a thing left on the ground.
fn played(data: &Data) -> (World, TilePos) {
    let region = data.region("hushlands").unwrap();
    let (mut w, layout) = patch(1, data, region);
    let me = w.character().unwrap();
    let first = layout
        .lots
        .iter()
        .find(|l| !l.holdout)
        .expect("a building in the town");
    let at = first.door;
    let door = (0..4)
        .map(|k| [Dir8::N, Dir8::S, Dir8::W, Dir8::E][k])
        .find(|&d| matches!(w.grid.edge(at, d), Edge::Door { .. }))
        .expect("a front door is by the tile just inside it");
    w.grid.set_edge(at, door, Edge::door(false));
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
    (w, at)
}

#[test]
fn a_saved_patch_loads_as_the_same_world_and_goes_on_the_same() {
    let data = data();
    let region = data.region("hushlands").unwrap();
    let (mut a, _) = played(&data);
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
    let (w, door) = played(&data);
    let changed: Vec<_> = w
        .grid
        .changed_chunks()
        .into_iter()
        .map(|(at, _)| at)
        .collect();
    assert_eq!(changed, vec![door.chunk()], "the door's chunk, of 64");
    let bytes = dusk_gen::save(&w, region, b"");
    assert!(bytes.len() < 100_000, "{} bytes", bytes.len());
}

#[test]
fn what_isnt_a_save_of_this_world_is_refused() {
    let data = data();
    let region = data.region("hushlands").unwrap();
    let (w, _) = played(&data);
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

#[test]
fn a_claimed_bounty_stays_claimed_after_a_load() {
    use dusk_core::{Intent, TilePos};
    let data = data();
    let region = data.region("hushlands").unwrap();
    let (mut w, _) = patch(1, &data, region);
    let me = w.character().unwrap();
    let rare = w.rares().next().expect("a rare");
    while !w.actor(rare).unwrap().dead {
        w.wound(rare, WoundKind::Bite, Part::Torso);
    }
    // The abbey's counter, and a tile beside it to stand on: walk there and claim.
    let counter = (0..256)
        .flat_map(|x| (0..256).map(move |y| TilePos::new(x, y, 0)))
        .find(|&p| w.counter_at(p).is_some())
        .unwrap();
    w.intent(
        me,
        Intent::WalkTo {
            to: counter.offset(-1, 0),
            run: false,
        },
    );
    w.step(400);
    w.intent(me, Intent::Claim { at: counter, rare });
    w.step(40);
    assert!(w.bounty_claimed(rare), "claimed");
    let bytes = dusk_gen::save(&w, region, &[]);
    let loaded = dusk_gen::load(&data, &bytes).unwrap_or_else(|e| panic!("{e}"));
    assert!(loaded.world.bounty_claimed(rare));
    assert_eq!(loaded.world.name_of(rare), w.name_of(rare));
    assert_eq!(loaded.world.state_hash(), w.state_hash());
}

#[test]
fn a_fire_alight_in_a_stove_stays_alight_after_a_load_and_burns_down_the_same() {
    use dusk_core::{ActorKind, TilePos};
    let data = data();
    let region = data.region("hushlands").unwrap();
    let (mut w, _) = patch(1, &data, region);
    // A stove with none of the dead within 12 tiles, and a survivor beside it.
    let quiet = |w: &World, at: TilePos| {
        w.actors()
            .all(|(_, a)| a.dead || a.kind.is_character() || a.pos.dist2(at) > 144)
    };
    let stove = (0..256)
        .flat_map(|y| (0..256).map(move |x| TilePos::new(x, y, 0)))
        .find(|&p| w.hearth_reach(p) > 0 && quiet(&w, p))
        .expect("a quiet stove");
    let beside: Vec<TilePos> = [Dir8::N, Dir8::E, Dir8::S, Dir8::W]
        .into_iter()
        .filter(|&d| w.grid.edge(stove, d) == Edge::Open && w.grid.tile(stove.step(d)).walkable())
        .map(|d| stove.step(d))
        .collect();
    let me = beside
        .iter()
        .find_map(|&p| w.spawn(ActorKind::Character, p))
        .expect("room beside it");
    let item = |id: &str| w.defs.items.iter().position(|i| i.id == id).unwrap() as u16;
    let (firewood, torch) = (item("firewood"), item("torch"));
    // The logs kept by it, one on, and lit from the kit's torch.
    assert_eq!(w.store(stove)[0].def, firewood, "logs by the stove");
    w.intent(
        me,
        Intent::Take {
            at: stove,
            slot: 0,
            def: firewood,
        },
    );
    w.step(60);
    w.intent(me, Intent::Tend);
    w.step(40);
    let slot = w
        .actor(me)
        .unwrap()
        .items
        .iter()
        .position(|i| i.def == torch)
        .unwrap() as u16;
    w.intent(me, Intent::Light { slot, def: torch });
    w.step(40);
    w.intent(me, Intent::Tend);
    w.step(40);
    assert!(w.hearth(stove).lit, "alight");
    w.step(250);

    let bytes = dusk_gen::save(&w, region, &[]);
    let mut b = dusk_gen::load(&data, &bytes)
        .unwrap_or_else(|e| panic!("{e}"))
        .world;
    assert_eq!(b.hearth(stove), w.hearth(stove));
    assert_eq!(b.state_hash(), w.state_hash());
    w.step(500);
    b.step(500);
    assert_eq!(b.hearth(stove), w.hearth(stove), "burnt down the same");
    assert_eq!(b.state_hash(), w.state_hash());
}

#[test]
fn a_clerics_faith_the_turned_and_a_warded_door_stay_so_after_a_load() {
    let data = data();
    let region = data.region("hushlands").unwrap();
    let (mut w, _) = patch(1, &data, region);
    let me = w.character().unwrap();
    let talent = |t: u8| data.defs.talents[usize::from(t)].id.as_str();
    let cleric = data.defs.classes.iter().position(|c| c.id == "cleric");
    assert!(w.choose_class(me, cleric.unwrap() as u8));
    // Warding, from the talents offered a level at a time.
    while !w
        .actor(me)
        .unwrap()
        .talents
        .iter()
        .any(|&t| talent(t) == "warding")
    {
        let need = w.rules.deeds.levels[usize::from(w.level(me)) - 1];
        let have = w.actor(me).unwrap().deeds;
        w.gain_deeds(me, need - have);
        let offer = w.talent_offer(me);
        let pick = offer.iter().copied().find(|&t| talent(t) == "warding");
        assert!(w.choose_talent(me, pick.unwrap_or(offer[0])));
    }
    // A shambler near the gate, to turn; and a shut door on your north side, to ward.
    let at = w.actor(me).unwrap().pos;
    let shambler = data.defs.creature_kind("shambler").unwrap();
    let z = (1..4)
        .flat_map(|r| (-r..=r).flat_map(move |dy| (-r..=r).map(move |dx| (dx, dy))))
        .find_map(|(dx, dy)| w.spawn(shambler, at.offset(dx, dy)))
        .expect("room for one");
    w.grid.set_edge(at, Dir8::N, Edge::door(false));
    w.intent(me, Intent::Turn);
    w.step(20);
    assert!(w.is_turned(z), "turned");
    w.intent(me, Intent::Ward);
    w.step(50);
    assert!(w.is_warded(at, Dir8::N), "warded");
    assert!(w.faith(me) < w.max_faith(me), "faith spent");

    let bytes = dusk_gen::save(&w, region, &[]);
    let b = dusk_gen::load(&data, &bytes)
        .unwrap_or_else(|e| panic!("{e}"))
        .world;
    assert_eq!(b.faith(me), w.faith(me));
    assert!(b.is_turned(z) && b.is_warded(at, Dir8::N));
    assert_eq!(b.class_of(me).map(|c| c.id.as_str()), Some("cleric"));
    assert_eq!(b.state_hash(), w.state_hash());
}
