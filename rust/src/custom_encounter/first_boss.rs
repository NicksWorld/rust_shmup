use gdnative::prelude::*;

use super::generic_encounter::GenericEncounter;

#[derive(NativeClass, Default)]
#[inherit(Node2D)]
pub struct FirstBoss {}

#[methods]
impl FirstBoss {
    fn new(_owner: &Node2D) -> Self {
        Self::default()
    }
}

impl GenericEncounter for FirstBoss {
    fn activate(&mut self, owner: &Node2D) {
        owner.set_visible(true);
    }
    fn deactivate(&mut self, owner: &Node2D) {
        owner.set_visible(false);
    }

    fn has_ended(&self) -> bool {
        false
    }
    fn end_delay(&self) -> i64 {
        1000
    }

    fn tick(&mut self, _owner: &Node2D, _deltatime: f32) {

    }

    fn hit_enemy(&mut self, _owner: &Node2D, _pos: Vector2, _radius: u32) -> bool {
        true
    }
}