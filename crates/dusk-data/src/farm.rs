//! A region's farms (`[farms]`, for the generator): farmsteads out in the fields, each reached
//! by a track off the road.

use serde::Deserialize;

/// A region's farms, as the generator reads them.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FarmsFile {
    /// The prefab tag a farmstead is picked by.
    pub tag: String,
    /// How many a patch tries for (one that finds no room isn't there).
    pub count: u32,
    /// Tiles from the road to a farm's fence: the fewest and the most.
    pub off_road: (i32, i32),
    /// Tiles a farm keeps clear of every other lot, and of the river.
    pub gap: i32,
}

impl FarmsFile {
    pub(crate) fn problem(&self) -> Option<String> {
        let (near, far) = self.off_road;
        if self.count > 8 || near < 1 || near > far || self.gap < 0 {
            return Some(
                "farms: count is 0 to 8, off_road is [fewest, most] from 1 tile, gap 0 or more"
                    .into(),
            );
        }
        None
    }
}
