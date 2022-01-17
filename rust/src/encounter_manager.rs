use gdnative::api::Node2D;
use gdnative::api::OS;
use gdnative::export::user_data::LocalCellError;
use gdnative::prelude::*;

use crate::custom_encounter::first_boss::FirstBoss;
use crate::custom_encounter::generic_encounter::GenericEncounter;
use crate::encounter::Encounter;

pub enum EncounterType {
    GenericEncounter(TInstance<'static, Encounter, Shared>),
    FirstBoss(TInstance<'static, FirstBoss, Shared>),
}

impl EncounterType {
    fn map_mut<F, U>(&self, func: F)
        -> Result<U, LocalCellError>
        where F: FnOnce(&mut dyn GenericEncounter, &Node2D) -> U,
    {
        match self {
            EncounterType::GenericEncounter(x) => {
                x.map_mut(|x: &mut Encounter, node: TRef<Node2D>| {func(x as &mut dyn GenericEncounter, node.cast::<Node2D>().unwrap().as_ref())})
            },
            EncounterType::FirstBoss(x) => {
                x.map_mut(|x: &mut FirstBoss, node: TRef<Node2D>| {func(x as &mut dyn GenericEncounter, node.cast::<Node2D>().unwrap().as_ref())})
            }
        }
    }
}

#[derive(NativeClass, Default)]
#[inherit(Node2D)]
pub struct EncounterManager {
    // List of encounters to progress through
    encounters: Vec<EncounterType>,
    // The index for the currently active encounter
    active_encounter: usize,
    // Time when the encounter ended (for handling end delay)
    // -1 for not yet set
    encounter_end: i64,
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
            let instance =
                unsafe {child.get_node_as_instance::<Encounter>(".").map(|x| EncounterType::GenericEncounter(x))}
                .or(unsafe {child.get_node_as_instance::<FirstBoss>(".").map(|x| EncounterType::FirstBoss(x))})
                .unwrap();
            // TODO: Use `.or()` to implement for additional encounter types

            self.encounters.push(instance);
        }

        // Tell each encounter if it is active or inactive
        for i in 0..self.encounters.len() {
            if i == 0 {
                self.encounters[i]
                    .map_mut(|encounter: &mut dyn GenericEncounter, node: &Node2D| {
                        encounter.activate(node);
                    }).unwrap();
            } else {
                self.encounters[i]
                    .map_mut(|encounter: &mut dyn GenericEncounter, node: &Node2D| {
                        encounter.deactivate(node);
                    }).unwrap();
            }
        }
    }

    // Tick the current encounter and check if ready to progress
    #[export]
    fn _process(&mut self, _owner: &Node2D, deltatime: f32) {
        if self.encounters.len() > self.active_encounter {
            self.encounters[self.active_encounter]
                .map_mut(|encounter: &mut dyn GenericEncounter, node: &Node2D| {
                    if encounter.has_ended() {
                        if OS::godot_singleton().get_ticks_msec() - encounter.end_delay() >= self.encounter_end {
                            encounter.activate(node);
                            self.active_encounter += 1;
                            self.encounter_end = -1;
                            if self.encounters.len() > self.active_encounter {
                                self.encounters[self.active_encounter]
                                    .map_mut(| encounter: &mut dyn GenericEncounter, node: &Node2D| {
                                        encounter.activate(node);
                                    })
                                    .unwrap();
                            }
                        } else {
                            if self.encounter_end == -1 {
                                self.encounter_end = OS::godot_singleton().get_ticks_msec();
                                encounter.tick(node, deltatime);
                            }
                        }
                    } else {
                        encounter.tick(node, deltatime);
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
                .map_mut(|encounter: &mut dyn GenericEncounter, node: &Node2D| {
                    encounter.hit_enemy(node, position, radius)
                })
                .unwrap()
        } else {
            false
        }
    }
}
