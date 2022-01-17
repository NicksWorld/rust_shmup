use gdnative::prelude::*;

pub trait GenericEncounter {
    fn activate(&mut self, owner: &Node2D);
    fn deactivate(&mut self, owner: &Node2D);

    fn has_ended(&self) -> bool;
    fn end_delay(&self) -> i64;

    fn tick(&mut self, owner: &Node2D, deltatime: f32);

    fn hit_enemy(&mut self, owner: &Node2D, pos: Vector2, radius: u32) -> bool;
}