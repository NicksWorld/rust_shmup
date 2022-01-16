use gdnative::api::Node2D;
use gdnative::api::Position2D;
use gdnative::api::OS;
use gdnative::prelude::*;

use crate::bullet_manager::BulletManager;

use crate::enemy::generic_enemy::GenericEnemy;

use std::f32::consts::PI;

#[derive(NativeClass, Default)]
#[inherit(Node2D)]
pub struct Orb {
    // Switches the direction the attack rotates
    #[property(default = false)]
    rotate_direction: bool,

    // Primary attack status
    last_attack: i64, // Time of last attack (msec)
    attack_timeout_ms: i64, // Time between attacks (msec)
    
    bullet_speed: f32, // Speed at which the bullets spawned travel
    attack_offset: f32, // Angle offset of the attack, changes each attack

    // Metadata for trait GenericEnemy
    health: u32,
    enabled: bool,
    goal_position: Vector2,
}

#[methods]
impl Orb {
    fn new(_owner: &Node2D) -> Self {
        Self {
            rotate_direction: false,

            last_attack: 0,
            attack_timeout_ms: 500,
            
            bullet_speed: 50.0,
            attack_offset: 0.0,

            health: 1,
            enabled: false,
            goal_position: Vector2::new(0.0, 0.0),
        }
    }

    #[export]
    fn _ready(&mut self, owner: &Node2D) {
        // Fetch the position of the Goal, where the Orb moves to before attacking
        let pos = unsafe { owner.get_node_as::<Position2D>("./Goal").unwrap() };
        self.goal_position = pos.global_position();
    }
}

impl GenericEnemy for Orb {
    // Radius of the hitbox
    const HITBOX_SIZE: u32 = 9;

    fn tick(
        &mut self,
        owner: &Node2D,
        bullet_handler: &TInstance<'static, BulletManager, Shared>,
        _player_pos: Vector2,
        _deltatime: f32,
    ) {
        // Prevent ticking if not enabled
        if !self.enabled {
            return;
        }

        // Handle primary attack
        let now = OS::godot_singleton().get_ticks_msec();
        if now - self.last_attack > self.attack_timeout_ms {
            for i in 0..9 {
                let angle = ((i as f32 * 40.0) + self.attack_offset) * PI / 180.0;

                let pos = owner.get_global_transform().origin;

                bullet_handler
                    .map_mut(|x: &mut BulletManager, node: TRef<Node2D>| {
                        x.spawn_bullet(
                            node.as_ref(),
                            "orb_bullet".to_string(),
                            pos.x, // + angle.cos() * 15.0,
                            pos.y, // + angle.sin() * 15.0,
                            angle.cos() * self.bullet_speed,
                            angle.sin() * self.bullet_speed,
                        )
                    })
                    .unwrap();
            }

            self.last_attack = now;

            if self.rotate_direction {
                self.attack_offset = (self.attack_offset + 5.0) % 360.0;
            } else {
                self.attack_offset = (self.attack_offset - 5.0) % 360.0;
            }
        }
    }

    // Decrement health when hit by a bullet
    // Disable when health reaches 0
    fn hit(&mut self, owner: &Node2D) -> bool {
        if self.health != 0 {
            self.health -= 1;

            if self.health == 0 {
                self.enabled = false;
                owner.set_visible(false);
            }

            true
        } else {
            false
        }
    }

    // Metadata for EncounterManager
    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn goal_position(&self, _owner: &Node2D) -> Vector2 {
        self.goal_position
    }

    fn is_killed(&self) -> bool {
        self.health == 0
    }
}
