//! A generational arena: values in slots, ids that stay unique when a slot is reused, and
//! iteration in slot order (the same order on every run).

/// An id into an [`Arena`]. An id whose value was removed never finds the slot's next value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub struct Id {
    index: u32,
    generation: u32,
}

impl Id {
    /// The slot. Steady for as long as the value lives; handy for staggering work by id.
    pub fn index(self) -> usize {
        self.index as usize
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
enum Slot<T> {
    Full { generation: u32, value: T },
    Free { generation: u32 },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub struct Arena<T> {
    slots: Vec<Slot<T>>,
    free: Vec<u32>,
    len: usize,
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Arena {
            slots: Vec::new(),
            free: Vec::new(),
            len: 0,
        }
    }
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn insert(&mut self, value: T) -> Id {
        self.len += 1;
        if let Some(index) = self.free.pop() {
            let generation = match self.slots[index as usize] {
                Slot::Free { generation } => generation.wrapping_add(1),
                Slot::Full { .. } => unreachable!("a slot on the free list is full"),
            };
            self.slots[index as usize] = Slot::Full { generation, value };
            Id { index, generation }
        } else {
            let index = self.slots.len() as u32;
            self.slots.push(Slot::Full {
                generation: 0,
                value,
            });
            Id {
                index,
                generation: 0,
            }
        }
    }

    pub fn remove(&mut self, id: Id) -> Option<T> {
        let slot = self.slots.get_mut(id.index())?;
        if !matches!(slot, Slot::Full { generation, .. } if *generation == id.generation) {
            return None;
        }
        let old = std::mem::replace(
            slot,
            Slot::Free {
                generation: id.generation,
            },
        );
        self.free.push(id.index);
        self.len -= 1;
        match old {
            Slot::Full { value, .. } => Some(value),
            Slot::Free { .. } => None,
        }
    }

    pub fn get(&self, id: Id) -> Option<&T> {
        match self.slots.get(id.index())? {
            Slot::Full { generation, value } if *generation == id.generation => Some(value),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, id: Id) -> Option<&mut T> {
        match self.slots.get_mut(id.index())? {
            Slot::Full { generation, value } if *generation == id.generation => Some(value),
            _ => None,
        }
    }

    /// Every id and value, in slot order.
    pub fn iter(&self) -> impl Iterator<Item = (Id, &T)> {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(i, slot)| match slot {
                Slot::Full { generation, value } => Some((
                    Id {
                        index: i as u32,
                        generation: *generation,
                    },
                    value,
                )),
                Slot::Free { .. } => None,
            })
    }

    /// Every id, in slot order.
    pub fn ids(&self) -> Vec<Id> {
        self.iter().map(|(id, _)| id).collect()
    }
}
