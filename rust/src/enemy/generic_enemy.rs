use gdnative::prelude::*;

use crate::bullet_manager::BulletManager;

pub trait GenericEnemy: NativeClass {
    // Used to determine if a Player's bullet has hit
    const HITBOX_SIZE: u32;

    // Function called for the enemy to perform its actions
    // as well as spawn bullets
    fn tick(
        &mut self,
        owner: &Self::Base,
        bullet_handler: &TInstance<'static, BulletManager, Shared>,
        player_pos: Vector2,
        deltatime: f32,
    );

    // Called when the enemy was hit by a bullet.
    // Enemy should decrease health, or return false
    // if invulnerable.
    fn hit(&mut self, owner: &Self::Base) -> bool;

    // Check if the enemy is active
    // Active if:
    // - Moved into position & Alive
    // Inactive if:
    // - Moving into position
    // - Has been killed
    // - Moved off-screen
    fn is_enabled(&self) -> bool;
    // Used to update the enabled status by the EncounterManager
    fn set_enabled(&mut self, enable: bool);

    // Tells the EncounterManager if the enemy has been killed.
    // Used in determining if the encounter is ready to end.
    fn is_killed(&self) -> bool;

    // Returns the position the EncounterManager moves the
    // enemy towards before enabling.
    fn goal_position(&self, owner: &Self::Base) -> Vector2;
}
