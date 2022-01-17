use gdnative::api::Node2D;
use gdnative::api::OS;
use gdnative::prelude::*;

use crate::bullet_manager::BulletManager;

#[derive(NativeClass, Default)]
#[inherit(Node2D)]
pub struct Player {
    #[property(default = 120)]
    pub speed: u32,
    #[property(default = 90)]
    shoot_timeout_ms: u32,
    #[property(default = 300.0)]
    primary_speed: f32,
    #[property(default = 1000)]
    hit_invulnerability_ms: i64,

    bullet_manager: Option<TInstance<'static, BulletManager, Shared>>,

    last_attack: i64,
    last_hit: i64, // -1 for never

    invulnerability_anim: u8,
}

#[methods]
impl Player {
    fn new(_owner: &Node2D) -> Self {
        Self {
            speed: 120,
            shoot_timeout_ms: 90,
            primary_speed: 300.0,
            bullet_manager: None,
            hit_invulnerability_ms: 1000,

            last_attack: 0,
            last_hit: -1,

            invulnerability_anim: 0,
        }
    }

    // Called when bullet hits the player's hitbox
    // Returning true deletes the bullet, Returning false persists it
    #[export]
    pub fn hit(&mut self, _owner: &Node2D) -> bool {
        if OS::godot_singleton().get_ticks_msec() - self.last_hit > self.hit_invulnerability_ms {
            self.last_hit = OS::godot_singleton().get_ticks_msec();
            true
        } else {
            false
        }
    }

    // Called when the game is ready to start
    #[export]
    fn _ready(&mut self, owner: &Node2D) {
        // Store a copy of the bullet manager in order to shoot bullets
        self.bullet_manager = unsafe { owner.get_node_as_instance::<BulletManager>("../Bullets") };
    }

    #[export]
    fn _process(&mut self, owner: &Node2D, delta: f32) {
        let input = Input::godot_singleton();

        // Calculate the position change for the tick
        let mut velocity = Vector2::new(0.0, 0.0);
        if Input::is_action_pressed(input, "move_up", false) {
            velocity.y -= 1.0;
        }
        if Input::is_action_pressed(input, "move_down", false) {
            velocity.y += 1.0;
        }
        if Input::is_action_pressed(input, "move_left", false) {
            velocity.x -= 1.0;
        }
        if Input::is_action_pressed(input, "move_right", false) {
            velocity.x += 1.0;
        }

        let change = velocity * self.speed as f32 * delta;
        owner.set_global_position(owner.global_position() + change);

        let now = OS::godot_singleton().get_ticks_msec();

        // Manage firing of bullets
        if Input::is_action_pressed(input, "shoot_1", false)
            && (now - self.last_attack) > self.shoot_timeout_ms as i64
        {
            self.last_attack = now;
            let pos = owner.global_position();
            self.bullet_manager
                .as_ref()
                .unwrap()
                .map_mut(|x: &mut BulletManager, node: TRef<Node2D>| {
                    // Spawn the set of bullets for the primary fire
                    x.spawn_bullet(
                        &*node,
                        "player_primary_03".to_string(),
                        pos.x - 12.0,
                        pos.y - 6.0,
                        -20.0,
                        -self.primary_speed,
                    );
                    x.spawn_bullet(
                        &*node,
                        "player_primary_02".to_string(),
                        pos.x - 10.0,
                        pos.y - 9.0,
                        -10.0,
                        -self.primary_speed,
                    );
                    x.spawn_bullet(
                        &*node,
                        "player_primary_01".to_string(),
                        pos.x - 4.0,
                        pos.y - 19.0,
                        0.0,
                        -self.primary_speed,
                    );
                    x.spawn_bullet(
                        &*node,
                        "player_primary_02".to_string(),
                        pos.x + 5.0,
                        pos.y - 9.0,
                        10.0,
                        -self.primary_speed,
                    );
                    x.spawn_bullet(
                        &*node,
                        "player_primary_03".to_string(),
                        pos.x + 11.0,
                        pos.y - 6.0,
                        20.0,
                        -self.primary_speed,
                    );
                })
                .unwrap();
        }

        // Animate invulnerability with toggling visibility
        if OS::godot_singleton().get_ticks_msec() - self.last_hit <= self.hit_invulnerability_ms {
            self.invulnerability_anim = (self.invulnerability_anim + 1) % 4;
            if self.invulnerability_anim <= 1 {
                owner.set_visible(true);
            } else {
                owner.set_visible(false);
            }
        } else if self.invulnerability_anim > 1 {
            owner.set_visible(true);
            self.invulnerability_anim = 0;
        }
    }
}
