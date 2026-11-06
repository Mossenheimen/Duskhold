use dusk_core::Dir8;

#[test]
fn nearest_heading_follows_the_angle_not_just_the_signs() {
    assert_eq!(Dir8::nearest(5, 1), Some(Dir8::E));
    assert_eq!(
        Dir8::toward(5, 1),
        Some(Dir8::SE),
        "toward only looks at the signs"
    );
    assert_eq!(Dir8::nearest(1, 5), Some(Dir8::S));
    assert_eq!(Dir8::nearest(3, 3), Some(Dir8::SE));
    assert_eq!(Dir8::nearest(-3, -2), Some(Dir8::NW));
    assert_eq!(Dir8::nearest(-4, 1), Some(Dir8::W));
    assert_eq!(Dir8::nearest(0, 0), None);
}

#[test]
fn turning_wraps_round() {
    assert_eq!(Dir8::N.turned(-1), Dir8::NW);
    assert_eq!(Dir8::NW.turned(1), Dir8::N);
    assert_eq!(Dir8::E.turned(4), Dir8::W);
}
