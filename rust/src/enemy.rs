use gdnative::prelude::InitHandle;

pub mod generic_enemy;

pub mod orb;
pub mod small_orb;

pub fn register(handle: &InitHandle) {
        handle.add_class::<orb::Orb>();
        handle.add_class::<small_orb::SmallOrb>();
}