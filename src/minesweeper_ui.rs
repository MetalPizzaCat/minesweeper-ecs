/**This file contains functions for interacting with ui, 
 * that do not have direct effect on gameplay
 */

use game_oxide_framework::components::*;
use specs::{Entity, Builder, World, WorldExt};
use nalgebra::Vector2;
use game_oxide_framework::layers;

///Updates numbers in the segmented display
pub fn update_segmented_display(world: &mut World, display: &Vec<Entity>, new_value: u32) {
    let mut values: [u32; 3] = [0; 3];
    values[2] = new_value % 10;
    values[1] = (new_value % 100 - values[2]) / 10;
    values[0] = (new_value - values[2] - values[1]) / 100;

    for i in 0..3 {
        if let Some(sprite) = world.write_component::<Sprite>().get_mut(display[i]) {
            sprite.name = values[i].to_string();
        }
    }
}

pub fn make_segmented_display(world: &mut World, position: Vector2<i32>) -> Vec<Entity> {
    let mut res: Vec<Entity> = Vec::new();
    for i in 0..3 {
        res.push(
            world
                .create_entity()
                .with(Sprite {
                    name: "0".to_owned(),
                    source_rect: None,
                    size: Vector2::new(44, 80),
                    visible: true,
                })
                .with(Position {
                    x: position.x + 44 * i,
                    y: position.y,
                })
                .with(Renderable::new(true, layers::RenderLayers::Gameplay as u32))
                .build(),
        );
    }
    res
}
