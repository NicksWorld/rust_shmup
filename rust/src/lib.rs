mod bullet_manager;
mod custom_encounter;
mod encounter;
mod encounter_manager;
mod enemy;
mod player;

use gdnative::prelude::*;

// Function that registers all exposed classes to Godot
fn init(handle: InitHandle) {
    // Manages the movement of all bullets
    handle.add_class::<bullet_manager::BulletManager>();

    // Manages each individual encounter (aka. Enemy management)
    handle.add_class::<encounter::Encounter>();
    // Manages switching between encounters to create a Stage
    handle.add_class::<encounter_manager::EncounterManager>();

    // Enemies
    enemy::register(&handle);

    // Encounters
    custom_encounter::register(&handle);

    // The player
    handle.add_class::<player::Player>();

    init_panic_hook();
}

// Panic handler (stolen from the gdnative examples)
pub fn init_panic_hook() {
    let old_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let loc_string;
        if let Some(location) = panic_info.location() {
            loc_string = format!("file '{}' at line {}", location.file(), location.line());
        } else {
            loc_string = "unknown location".to_owned()
        }

        let error_message;
        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            error_message = format!("[RUST] {}: panic occurred: {:?}", loc_string, s);
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            error_message = format!("[RUST] {}: panic occurred: {:?}", loc_string, s);
        } else {
            error_message = format!("[RUST] {}: unknown panic occurred", loc_string);
        }
        godot_error!("{}", error_message);
        (*(old_hook.as_ref()))(panic_info);

        unsafe {
            if let Some(gd_panic_hook) =
                gdnative::api::utils::autoload::<gdnative::api::Node>("rust_panic_hook")
            {
                gd_panic_hook.call(
                    "rust_panic_hook",
                    &[GodotString::from_str(error_message).to_variant()],
                );
            }
        }
    }));
}

// Initialize the GodotNative library
godot_init!(init);