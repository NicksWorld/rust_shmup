use gdnative::prelude::*;

pub mod generic_encounter;
pub mod first_boss;

pub fn register(handle: &InitHandle) {
    handle.add_class::<first_boss::FirstBoss>();
}