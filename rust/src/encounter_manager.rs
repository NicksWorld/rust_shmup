use gdnative::api::Node2D;
use gdnative::prelude::*;

use crate::encounter::Encounter;

#[derive(NativeClass, Default)]
#[inherit(Node2D)]
pub struct EncounterManager {
    // List of encounters to progress through
    encounters: Vec<TInstance<'static, Encounter, Shared>>,
    // The index for the currently active encounter
    active_encounter: usize,
}

#[methods]
impl EncounterManager {
    fn new(_owner: &Node2D) -> Self {
        Self::default()
    }

    #[export]
    fn _ready(&mut self, owner: &Node2D) {
        // Store a refrence to every Encounter stored in the children of Encounters
        for child in owner.get_children().iter() {
            let child = unsafe { child.to_object::<Node2D>().unwrap().assume_safe() };
            let instance = unsafe { child.get_node_as_instance::<Encounter>(".").unwrap() };

            self.encounters.push(instance);
        }

        // Tell each encounter if it is active or inactive
        for i in 0..self.encounters.len() {
            if i == 0 {
                self.encounters[i]
                    .map_mut(|encounter: &mut Encounter, node: TRef<Node2D>| {
                        encounter.set_active(node.as_ref())
                    })
                    .unwrap();
            } else {
                self.encounters[i]
                    .map_mut(|encounter: &mut Encounter, node: TRef<Node2D>| {
                        encounter.set_inactive(node.as_ref())
                    })
                    .unwrap();
            }
        }
    }

    // Tick the current encounter and check if ready to progress
    #[export]
    fn _process(&mut self, _owner: &Node2D, deltatime: f32) {
        if self.encounters.len() > self.active_encounter {
            self.encounters[self.active_encounter]
                .map_mut(|encounter: &mut Encounter, node: TRef<Node2D>| {
                    if encounter.ended {
                        encounter.set_inactive(node.as_ref());
                        self.active_encounter += 1;
                        if self.encounters.len() > self.active_encounter {
                            self.encounters[self.active_encounter]
                                .map_mut(| encounter: &mut Encounter, node: TRef<Node2D>| {
                                    encounter.set_active(node.as_ref());
                                })
                                .unwrap();
                        }
                    } else {
                        encounter.tick(node.as_ref(), deltatime);
                    }
                })
                .unwrap();
        } else {
            // TODO: Link with some sort of Stage manger
            godot_warn!("No more encounters!");
        }
    }

    // Forward calls to the current encounter
    pub fn hit_enemy(&mut self, _owner: &Node2D, position: Vector2, radius: u32) -> bool {
        if self.encounters.len() > self.active_encounter {
            self.encounters[self.active_encounter]
                .map_mut(|encounter: &mut Encounter, node: TRef<Node2D>| {
                    encounter.hit_enemy(node.as_ref(), position, radius)
                })
                .unwrap()
        } else {
            false
        }
    }
}
