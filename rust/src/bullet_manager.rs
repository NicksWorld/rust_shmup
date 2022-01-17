use gdnative::api::Node2D;
use gdnative::prelude::*;

use std::collections::HashMap;

use crate::encounter_manager::EncounterManager;
use crate::player::Player;

// Storage type for tracking bullet sprites
struct Bullet {
    node: Ref<Node2D, Shared>,
    dx: f32,
    dy: f32,
}

// Storage type for seperating types of bullets
struct BulletEntry {
    alive: Vec<Bullet>,
    dead: Vec<Bullet>,
    amount: i32,
    radius: u32,
    scene: Ref<PackedScene, Shared>,
}

// Provides a list of bullet types
// Adding to this list will add to the properties as well
pub fn bullet_types() -> Vec<&'static str> {
    vec![
        // Player bullets
        "player_primary_01",
        "player_primary_02",
        "player_primary_03",
        // Orb bullets
        "orb_bullet",
    ]
}

#[derive(NativeClass, Default)]
#[inherit(Node2D)]
#[register_with(Self::register)]
pub struct BulletManager {
    bullets: HashMap<String, BulletEntry>,

    enemy_manager: Option<TInstance<'static, EncounterManager, Shared>>,
    player: Option<TInstance<'static, Player, Shared>>,
}

#[methods]
impl BulletManager {
    // Produces the list of properties needed to configure the
    // packed scene and quantity of each bullet
    fn register(builder: &ClassBuilder<Self>) {
        for bullet_type in bullet_types() {
            builder
                .property(&format!("bullet_scenes/{bullet_type}"))
                .with_ref_getter(move |this: &BulletManager, _owner: TRef<Node2D>| {
                    let res = &this.bullets.get(bullet_type).unwrap().scene;
                    res
                })
                .with_setter(move |this: &mut BulletManager, _owner: TRef<Node2D>, v| {
                    this.bullets
                        .entry(bullet_type.to_string())
                        .or_insert(BulletEntry {
                            amount: 0,
                            radius: 0,
                            scene: v,
                            alive: vec![],
                            dead: vec![],
                        });
                })
                .done();

            builder
                .property(&format!("bullet_amounts/{bullet_type}"))
                .with_default(0)
                .with_ref_getter(move |this: &BulletManager, _owner: TRef<Node2D>| {
                    let res = &this.bullets.get(bullet_type).unwrap().amount;
                    res
                })
                .with_setter(move |this: &mut BulletManager, _owner: TRef<Node2D>, v| {
                    let entry = this.bullets.get_mut(&bullet_type.to_string());
                    if entry.is_some() {
                        (*entry.unwrap()).amount = v;
                    } else {
                        godot_warn!("Attempt to set bullet amount for {bullet_type} without scene");
                    }
                })
                .done();

            builder
                .property(&format!("bullet_radius/{bullet_type}"))
                .with_default(0)
                .with_ref_getter(move |this: &BulletManager, _owner: TRef<Node2D>| {
                    let res = &this.bullets.get(bullet_type).unwrap().radius;
                    res
                })
                .with_setter(move |this: &mut BulletManager, _owner: TRef<Node2D>, v| {
                    let entry = this.bullets.get_mut(&bullet_type.to_string());
                    if entry.is_some() {
                        (*entry.unwrap()).radius = v;
                    } else {
                        godot_warn!("Attempt to set bullet radius for {bullet_type} without scene");
                    }
                })
                .done();
        }
    }

    fn new(_owner: &Node2D) -> Self {
        Self::default()
    }

    #[export]
    pub fn _ready(&mut self, owner: &Node2D) {
        // Take a refrence to the EncountersHandler to manage checking for bullet collision
        // with enemies.
        self.enemy_manager =
            unsafe { owner.get_node_as_instance::<EncounterManager>("../Encounters") };
        // Take a refrence to the Player in order to manage bullet collision with the player
        self.player = unsafe { owner.get_node_as_instance::<Player>("../Player") };

        // Iterate through the bullet types and initialize the bullet sprites
        for bullet_type in bullet_types() {
            let bullet_info = self.bullets.get_mut(bullet_type).unwrap();

            for _ in 0..bullet_info.amount {
                // Instance a "scene" to create a bullet sprite
                let bullet: Ref<Node2D, _> = BulletManager::instance_scene(&bullet_info.scene);
                bullet.set_visible(false);

                let bullet = bullet.into_shared();
                owner.add_child(bullet, false);

                // Insert the bullet into the list of managed bullets
                let bullet = Bullet {
                    dx: 0.0,
                    dy: 0.0,
                    node: bullet,
                };

                (*bullet_info).dead.push(bullet);
            }
        }
    }

    #[export]
    pub fn _process(&mut self, _owner: &Node2D, deltatime: f32) {
        let enemy_manager = self.enemy_manager.as_ref().unwrap();
        // Request the player's position to assist in collision calculations
        let player_pos = self
            .player
            .as_ref()
            .unwrap()
            .map(|_: &Player, node: TRef<Node2D>| node.global_position())
            .unwrap();
        for bullet_type in bullet_types() {
            // Check if the bullet is from the player
            // allows collision with enemies, disallows collision with player
            let is_player = bullet_type.starts_with("player");

            // List of indexes to transfer from living to dead
            let mut to_remove = vec![];
            // Refrence to the configuration for bullet type
            let bullet_info = self.bullets.get_mut(bullet_type).unwrap();

            for i in 0..bullet_info.alive.len() {
                let bullet = &bullet_info.alive[i];
                let velocity = Vector2::new(bullet.dx, bullet.dy) * deltatime;

                let node = unsafe { bullet.node.assume_safe() };

                node.set_global_position(node.global_position() + velocity);

                // Check for collisions (left screen, hit player, hit enemy)
                let pos = node.global_position();
                if pos.x < 0.0 || pos.y < 0.0 || pos.x > 480.0 || pos.y > 270.0 {
                    node.set_visible(false);
                    to_remove.push(i);
                } else if is_player {
                    if enemy_manager
                        .map_mut(|x: &mut EncounterManager, node: TRef<Node2D>| {
                            // Request the Encounter to check for bullet collisions
                            x.hit_enemy(node.as_ref(), pos, bullet_info.radius)
                        })
                        .unwrap()
                    {
                        // Push the bullet back into the queue to be reused
                        node.set_visible(false);
                        to_remove.push(i);
                    }
                } else {
                    // Check if the player is within 4 + bullet_radius of the bullet
                    if (player_pos.x - pos.x).powf(2.0) + (player_pos.y - pos.y).powf(2.0)
                        <= (4.0 + bullet_info.radius as f32).powf(2.0)
                    {
                        if self.player
                            .as_ref()
                            .unwrap()
                            .map_mut(|x, node| x.hit(node.as_ref()))
                            .unwrap() {
                            node.set_visible(false);
                            to_remove.push(i);
                        }
                    }
                }
            }

            // Swap bullets into the dead queue
            for i in to_remove.iter().rev() {
                // Use swap_remove on a backwards list of indexes
                // to not change order as well as increase performance
                // if many bullets are removed at once
                bullet_info.dead.push(bullet_info.alive.swap_remove(*i));
            }
        }
    }

    // Called by Player and GenericEnemy to spawn a bullet
    #[export]
    pub fn spawn_bullet(
        &mut self,
        _owner: &Node2D,
        kind: String,
        x: f32,
        y: f32,
        dx: f32,
        dy: f32,
    ) {
        let bullets = self.bullets.get_mut(&kind).unwrap();
        // Fetch a bullet from the dead list and provide parameters
        let mut bullet = bullets.dead.pop().unwrap();
        bullet.dx = dx;
        bullet.dy = dy;

        let node = unsafe { bullet.node.assume_safe() };
        node.set_global_position(Vector2::new(x, y));
        node.set_visible(true);

        // Place the bullet into the living list to be ticked
        bullets.alive.push(bullet);
    }

    // Creates a new instance of a "scene"
    fn instance_scene<Root>(scene: &Ref<PackedScene, Shared>) -> Ref<Root, Unique>
    where
        Root: gdnative::object::GodotObject<Memory = ManuallyManaged> + SubClass<Node>,
    {
        // Instantiate the scene with scene modification disabled
        let scene = unsafe { scene.assume_safe() };
        let instance = scene
            .instance(PackedScene::GEN_EDIT_STATE_DISABLED)
            .unwrap();

        let instance = unsafe { instance.assume_unique() };

        // Cast the new instance for use as a Node2D
        instance.try_cast::<Root>().unwrap()
    }
}
