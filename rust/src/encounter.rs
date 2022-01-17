use gdnative::prelude::*;
use gdnative::api::OS;
use gdnative::export::user_data::{Map, MapMut};

use crate::bullet_manager::BulletManager;
use crate::custom_encounter::generic_encounter::GenericEncounter;
use crate::enemy::*;
use generic_enemy::GenericEnemy;
use crate::player::Player;

// TODO: Change implementation to be ontop of GenericEncounter
#[derive(NativeClass, Default)]
#[inherit(Node2D)]
pub struct Encounter {
    // Enemy variants
    // TODO: Try replacing with HashMap<TypeId, Vec<Box<Any>>>
    // where Box<Any> is a TInstance<'static, T: NativeClass, Shared>
    // To allow for the addition of new enemies with a change to a function
    // that returns a Vec<(TypeId, String)>, allowing properties to be created
    // which allow enabling and disabling enemy types per encounter
    // as well as future-proofing the code written here for new enemy variants
    orbs: Vec<TInstance<'static, orb::Orb, Shared>>,
    small_orbs: Vec<TInstance<'static, small_orb::SmallOrb, Shared>>,

    // Bullet manager for ticking enemies
    bullet_manager: Option<TInstance<'static, BulletManager, Shared>>,
    // Player for tracking position for collisions
    player: Option<TInstance<'static, Player, Shared>>,

    // Time (secs) that the encounter will last before timing out
    // -1 = infinite
    #[property(default = -1)]
    encounter_length: i64,
    // Time the encounter started
    encounter_starttime: i64,
    // Time to wait after encounter completion (Handled by EncounterManager)
    #[property(default = 0)]
    encounter_end_delay: i64,
    // Wether the encounter has completed
    ended: bool,
}

#[methods]
impl Encounter {
    fn new(_owner: &Node2D) -> Self {
        Self {
            encounter_length: -1,
            ..Default::default()
        }
    }

    // Adds refrences to each enemy in the encounter to
    // their field in the struct
    fn process_children<T>(
        owner: &Node2D,
        name: &'static str,
        list: &mut Vec<TInstance<'static, T, Shared>>,
    ) where
        T: NativeClass,
        <T as NativeClass>::Base: SubClass<Node>,
    {
        for child in unsafe { owner.get_node_as::<Node2D>(name).unwrap() }
            .get_children()
            .iter()
        {
            let child = unsafe { child.to_object::<Node2D>().unwrap().assume_safe() };
            let instance = unsafe { child.get_node_as_instance::<T>(".").unwrap() };

            list.push(instance);
        }
    }

    #[export]
    fn _ready(&mut self, owner: &Node2D) {
        self.bullet_manager =
            unsafe { owner.get_node_as_instance::<BulletManager>("../../Bullets") };
        self.player = unsafe { owner.get_node_as_instance::<Player>("../../Player") };

        // Populate the enemy list
        Encounter::process_children(owner, "Orbs", &mut self.orbs);
        Encounter::process_children(owner, "SmallOrbs", &mut self.small_orbs);
    }

    fn process_enemies<T>(
        items: &Vec<TInstance<'static, T, Shared>>,
        bullet_manager: &TInstance<'static, BulletManager, Shared>,
        player_pos: Vector2,
        deltatime: f32,
    ) -> usize
    where
        T: GenericEnemy + NativeClass,
        <T as NativeClass>::UserData: MapMut,
        Node2D: SubClass<<T as NativeClass>::Base>,
    {
        let mut remaining_enemies = items.len();
        for enemy in items {
            enemy
                .map_mut(|x: &mut T, node: TRef<T::Base>| {
                    if x.is_enabled() {
                        x.tick(node.as_ref(), bullet_manager, player_pos, deltatime)
                    } else if x.is_killed() {
                        remaining_enemies -= 1;
                    } else {
                        let node2d = node.cast::<Node2D>().unwrap();
                        let pos = node2d.global_position();
                        let goal = x.goal_position(node.as_ref());
                        let movement_speed = 80.0;

                        let angle = (-pos.y + goal.y).atan2(-pos.x + goal.x);
                        let mut new_pos = Vector2::new(pos.x, pos.y)
                            + (Vector2::new(angle.cos(), angle.sin()) * movement_speed * deltatime);
                        if (new_pos.x - goal.x).powf(2.0) + (new_pos.y - goal.y).powf(2.0)
                            <= 400.0 * deltatime
                        {
                            new_pos = goal;
                            x.set_enabled(true);
                        }

                        node2d.set_global_position(new_pos);
                    }
                })
                .unwrap();
        }

        remaining_enemies
    }

    fn process_hits<T>(
        items: &Vec<TInstance<'static, T, Shared>>,
        position: Vector2,
        radius: u32,
    ) -> bool
    where
        T: NativeClass + GenericEnemy,
        <T as NativeClass>::UserData: Map + MapMut,
        Node2D: SubClass<<T as NativeClass>::Base>,
    {
        for enemy in items {
            let pos = enemy
                .map(|_, node: TRef<T::Base>| node.cast::<Node2D>().unwrap().global_position())
                .unwrap();
            if (pos.x - position.x).powf(2.0) + (pos.y - position.y).powf(2.0)
                <= (radius as f32 + 5.0).powf(2.0)
            {
                if enemy
                    .map_mut(|x: &mut T, node: TRef<T::Base>| x.hit(node.as_ref()))
                    .unwrap()
                {
                    return true;
                }
            }
        }

        false
    }
}

impl GenericEncounter for Encounter {
    fn activate(&mut self, owner: &Node2D) {
        owner.set_visible(true);
        self.encounter_starttime = OS::godot_singleton().get_ticks_msec();
    }
    fn deactivate(&mut self, owner: &Node2D) {
        owner.set_visible(false);
    }

    fn has_ended(&self) -> bool {
        self.ended
    }
    fn end_delay(&self) -> i64 {
        self.encounter_end_delay
    }

    fn tick(&mut self, _owner: &Node2D, deltatime: f32) {
        let player_pos = self
            .player
            .as_ref()
            .unwrap()
            .map(|_x, node| node.get_global_transform().origin)
            .unwrap();

        let bullet_manager = self.bullet_manager.as_ref().unwrap();

        let remaining_enemies =
            Encounter::process_enemies(&self.orbs, bullet_manager, player_pos, deltatime)
                + Encounter::process_enemies(
                    &self.small_orbs,
                    bullet_manager,
                    player_pos,
                    deltatime,
                );

        // TODO: Additionally wait for non-player bullets to be destroyed, allowing for a clear playspace
        if remaining_enemies == 0 {
            self.ended = true;
        } else if self.encounter_length != -1 && OS::godot_singleton().get_ticks_msec() - self.encounter_starttime
            >= self.encounter_length
        {
            self.ended = true;
        }
    }

    fn hit_enemy(&mut self, _owner: &Node2D, pos: Vector2, radius: u32) -> bool {
        return Encounter::process_hits(&self.orbs, pos, radius)
            || Encounter::process_hits(&self.small_orbs, pos, radius);
    }
}