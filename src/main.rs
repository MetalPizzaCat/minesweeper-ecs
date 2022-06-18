use game_oxide_framework::*;
use game_oxide_framework::{components::*, render::*, texture_manager::*};
use nalgebra::{Vector2, Vector4};
use sdl2::event::Event;
use specs::{
    Builder, Component, DispatcherBuilder, Entity, NullStorage, VecStorage, World, WorldExt,
};

use rand::distributions::{Distribution, Uniform};
use std::time::SystemTime;
pub mod assets;
pub mod minesweeper_ui;
use minesweeper_ui::*;

///Defines the parent of the drop down menu items. this is the thing that gets unwrapped
#[derive(Clone, Debug, PartialEq, Component, Default)]
#[storage(VecStorage)]
pub struct Tile {
    pub position: Vector2<usize>,
    pub revealed: bool,
}
#[derive(Default, Clone)]
struct Field {
    value: i32,
    bomb: bool,
    border: bool,
    revealed: bool,
    flagged: bool,
}

#[derive(Component, Default, Clone)]
#[storage(NullStorage)]
struct FaceButton;

enum GameState {
    ///Game is still happening
    Active,
    ///Player is browsing menues
    Setup,
    ///Player has either won or lost
    Ended,
}

fn generate_grid(area_size: usize, bomb_count: u32) -> Vec<Vec<Field>> {
    let mut rng = rand::thread_rng();
    //generate default grid
    let mut grid: Vec<Vec<Field>> = vec![vec![Field::default(); area_size]; area_size];
    let mut bombs: Vec<Vector2<usize>> = Vec::new();

    let mut bomb_count = bomb_count;
    let die = Uniform::from(0..area_size);

    //generate all bombs
    //we simply pick a random position and then if the position is taken
    // we move diagonally to the left bottom until we hit a good spot
    //if we reach the corner we move the line down and start from almost top right corner doing the same thing
    while bomb_count > 0 {
        let mut point: Vector2<usize> = Vector2::new(die.sample(&mut rng), die.sample(&mut rng));
        while grid[point.x][point.y].bomb {
            //to prevent going over the border because line was randomly generated
            if point.x >= area_size || point.y >= area_size {
                point.x = 1;
                point.y = 0;
            } else {
                point.x += 1;
                point.y += 1
            }
        }
        grid[point.x][point.y].bomb = true;
        bombs.push(point);
        bomb_count -= 1;
    }
    //calculate values for bombs
    for i in 0..area_size {
        for j in 0..area_size {
            if grid[i][j].bomb {
                //this weird loop below just represents going over a 3x3 square
                //with checks to prevent going over borders
                for a in -1i32..=1 {
                    let vert: i32 = i as i32 + a;
                    if vert >= area_size as i32 || vert < 0 {
                        continue;
                    }
                    for b in -1i32..=1 {
                        let hor: i32 = j as i32 + b;
                        if hor >= area_size as i32 || hor < 0 {
                            continue;
                        }
                        //we ignore the fact that the tile is a bomb and still add value
                        //because bomb with value is still a bomb, duh
                        grid[vert as usize][hor as usize].value += 1;
                        //mark this tile as border
                        if !grid[vert as usize][hor as usize].bomb {
                            grid[vert as usize][hor as usize].border = true;
                        }
                    }
                }
            }
        }
    }
    grid
}

///Reveal the tile and all neighboring 0 tiles using flood algorithm
fn reveal_block(
    point: Vector2<i32>,
    grid: &mut Vec<Vec<Field>>,
    world: &mut World,
    buttons: &Vec<Vec<Entity>>,
) {
    //simple border check
    if point.x < 0 || point.x >= grid.len() as i32 || point.y < 0 || point.y >= grid.len() as i32 {
        return;
    }
    let x = point.x as usize;
    let y = point.y as usize;
    //don't reveal bombs
    if grid[x][y].bomb || grid[x][y].revealed {
        return;
    }
    //update tile state so it would be drawn
    {
        if let Some(tile) = world.write_component::<Tile>().get_mut(buttons[x][y]) {
            tile.revealed = true;
        }
        if let Some(text) = world.write_component::<Text>().get_mut(buttons[x][y]) {
            text.visible = true;
        }
        if let Some(sprite) = world.write_component::<Sprite>().get_mut(buttons[x][y]) {
            if grid[x][y].bomb {
                sprite.name = "tile_bomb".to_owned();
            } else {
                sprite.name = "tile_".to_owned() + grid[x][y].value.to_string().as_str();
            }
        }
        //once we reveal tile it stops being a button
        world.write_component::<ui::Button>().remove(buttons[x][y]);
        //mark tile as visited
        grid[x][y].revealed = true;
    }
    //if this is a border we want to display the tile itself, but not go any further
    if grid[x][y].border {
        return;
    }
    //reveal neighbors
    reveal_block(point + Vector2::new(0, -1), grid, world, buttons);
    reveal_block(point + Vector2::new(-1, 0), grid, world, buttons);
    reveal_block(point + Vector2::new(0, 1), grid, world, buttons);
    reveal_block(point + Vector2::new(1, 0), grid, world, buttons);
}

///Attempts to flag a block
/// If fails returns false
fn flag_block(
    point: Vector2<i32>,
    grid: &mut Vec<Vec<Field>>,
    world: &mut World,
    buttons: &Vec<Vec<Entity>>,
    flag_count: &mut i32,
    total_mine_count: i32,
) -> bool {
    if point.x < 0 || point.x >= grid.len() as i32 || point.y < 0 || point.y >= grid.len() as i32 {
        return false;
    }

    let x = point.x as usize;
    let y = point.y as usize;
    //if tile was revealed then we either know it's not a bomb or we lost the game
    //no point in flagging it either way
    if grid[x][y].revealed {
        return false;
    }
    if !grid[x][y].flagged && *flag_count >= total_mine_count {
        return false;
    }

    if let Some(sprite) = world.write_component::<Sprite>().get_mut(buttons[x][y]) {
        if grid[x][y].flagged {
            sprite.name = "tile_default".to_owned();
        } else {
            sprite.name = "tile_flag".to_owned();
        }
    }
    if let Some(button) = world.write_component::<ui::Button>().get_mut(buttons[x][y]) {
        if !grid[x][y].flagged {
            button.hovered_over_texture_name = Some("tile_flag".to_owned());
            button.normal_texture_name = Some("tile_flag".to_owned());
        } else {
            button.hovered_over_texture_name = Some("tile_selected".to_owned());
            button.normal_texture_name = Some("tile_default".to_owned());
        }
    }
    grid[x][y].flagged = !grid[x][y].flagged;
    *flag_count += if grid[x][y].flagged { 1 } else { -1 };
    return true;
}

fn end_game(
    win: bool,
    world: &mut World,
    face: &Entity,
    buttons: &Vec<Vec<Entity>>,
    grid: &Vec<Vec<Field>>,
    area_size: usize,
) {
    if let Some(button) = world.write_component::<ui::Button>().get_mut(*face) {
        button.normal_texture_name = if win {
            Some("face_win".to_owned())
        } else {
            Some("face_loose".to_owned())
        }
    }
    //TODO: Replace with actual code
    if !win {
        //reveal all bombs
        for i in 0..area_size {
            for j in 0..area_size {
                if grid[i][j].bomb {
                    //update tile state so it would be drawn
                    {
                        if let Some(tile) = world.write_component::<Tile>().get_mut(buttons[i][j]) {
                            tile.revealed = true;
                        }
                        if let Some(text) = world.write_component::<Text>().get_mut(buttons[i][j]) {
                            text.visible = true;
                        }
                        if let Some(sprite) =
                            world.write_component::<Sprite>().get_mut(buttons[i][j])
                        {
                            sprite.name = "tile_bomb".to_owned();
                        }
                        //once we reveal tile it stops being a button
                        world.write_component::<ui::Button>().remove(buttons[i][j]);
                    }
                }
            }
        }
    }
}

///Checks if all mines have been flagged
fn check_mines(grid: &mut Vec<Vec<Field>>, area_size: usize) -> bool {
    for i in 0..area_size {
        for j in 0..area_size {
            if grid[i][j].bomb && !grid[i][j].flagged {
                return false;
            }
        }
    }
    true
}

fn make_text_box(
    world: &mut World,
    default_text: String,
    layer: layers::RenderLayers,
    location: Vector2<i32>,
) -> Entity {
    world
        .create_entity()
        .with(Position {
            x: location.x,
            y: location.y,
        })
        .with(Rectangle {
            width: 100,
            height: 50,
        })
        .with(Colored {
            color: sdl2::pixels::Color::RGB(0, 0, 0),
        })
        .with(Text {
            text: default_text,
            color: sdl2::pixels::Color::RED,
            visible: true,
            offset: Vector2::new(10, 0),
        })
        .with(Renderable::new(true, layer as u32))
        .build()
}

fn generate_game(
    world: &mut World,
    buttons: &mut Vec<Vec<Entity>>,
    mine_count: u32,
    area_size: usize,
    controls_panel_size: i32,
    bomb_display: &Vec<Entity>,
) -> Result<Vec<Vec<Field>>, String> {
    let mut grid = generate_grid(10, mine_count);

    if buttons.len() > 0 {
        for i in 0..area_size {
            for j in 0..area_size {
                if buttons[i].len() > 0 {
                    world
                        .delete_entity(buttons[i][j])
                        .map_err(|e| e.to_string())?;
                }
            }
        }
    }
    buttons.clear();

    //generate button entities
    for i in 0..area_size {
        buttons.push(Vec::new());
        for j in 0..area_size {
            buttons[i].push(
                ui::make_button_base(
                    world,
                    Vector2::new(j as i32 * 50 + 2, i as i32 * 50 + 2 + controls_panel_size),
                    Vector2::new(45, 45),
                    Some(ui::Button {
                        hovered_over: false,
                        hovered_over_texture_name: Some("tile_selected".to_owned()),
                        hovered_over_text: None,
                        hovered_over_color: Some(sdl2::pixels::Color::RGBA(255, 0, 255, 120)),
                        normal_texture_name: Some("tile_default".to_owned()),
                        normal_text: None,
                        normal_color: Some(sdl2::pixels::Color::RGBA(
                            if grid[i][j].border { 0 } else { 255 },
                            255,
                            255,
                            120,
                        )),
                    }),
                    sdl2::pixels::Color::RGBA(255, 255, 255, 120),
                    layers::RenderLayers::Menu,
                )
                .with(Tile {
                    position: Vector2::new(i, j),
                    revealed: false,
                })
                .with(Sprite {
                    name: "tile_default".to_owned(),
                    source_rect: Some(Vector4::new(0, 0, 16, 16)),
                    size: Vector2::new(50, 50),
                    visible: true,
                })
                .build(),
            );
        }
    }
    update_segmented_display(world, bomb_display, mine_count);
    Ok(grid)
}

fn main() -> Result<(), String> {
    let area_size: usize = 10;
    let controls_panel_size: u32 = 100;
    let (mut world, sdl, video_subsystem, ttf_context, mut canvas, mut game) = setup::setup(
        "Rust Minesweeper by MetalPizzaCat".to_owned(),
        Some(Vector2::new(
            area_size as u32 * 50,
            area_size as u32 * 50 + controls_panel_size,
        )),
    )?;
    let mut dispatcher = DispatcherBuilder::new()
        .with(ui::ButtonUpdateSystem, "button_update_system", &[])
        .build();
    ui::register_ui_components(&mut world);
    let mut event_pump = sdl.event_pump().unwrap();

    let mut texture_creator = canvas.texture_creator();
    let mut texture_manager = TextureManager::new(&texture_creator)?;
    assets::load_textures(&mut texture_manager)?;
    //register components necessary for ECS world to function
    world.register::<Tile>();
    world.register::<FaceButton>();
    world.insert(ui::MouseData::default());
    let font = ttf_context
        .load_font("./assets/fonts/Roboto-Medium.ttf", 50)
        .unwrap();

    let mut buttons: Vec<Vec<Entity>> = Vec::new();

    //game variables
    let total_mine_count = 10;
    let mut mines_left = total_mine_count;
    let mut flag_count: i32 = 0;
    let mut time: i32 = 0;
    let mut current_state: GameState = GameState::Active;

    
    world
        .create_entity()
        .with(Position { x: 0, y: 0 })
        .with(Rectangle {
            width: area_size as i32 * 50,
            height: controls_panel_size as i32,
        })
        .with(Colored {
            color: sdl2::pixels::Color::RGB(192, 192, 192),
        })
        .with(Renderable::new(true, layers::RenderLayers::Menu as u32))
        .build();

    let face = ui::make_button_base(
        &mut world,
        Vector2::new(
            area_size as i32 * 50 / 2 - 25,
            controls_panel_size as i32 / 2 - 25,
        ),
        Vector2::new(50, 50),
        Some(ui::Button {
            hovered_over: false,
            hovered_over_texture_name: Some("face_hover".to_owned()),
            hovered_over_text: None,
            hovered_over_color: None,
            normal_texture_name: Some("face_default".to_owned()),
            normal_text: None,
            normal_color: None,
        }),
        sdl2::pixels::Color::GREY,
        layers::RenderLayers::Gameplay,
    )
    .with(FaceButton)
    .with(Sprite {
        name: "face_default".to_owned(),
        source_rect: None,
        size: Vector2::new(50, 50),
        visible: true,
    })
    .build();

    let mine_display =
        make_segmented_display(&mut world, Vector2::new(area_size as i32 * 50 - 200, 10));
    let time_display = make_segmented_display(&mut world, Vector2::new(50, 10));
    let mut grid: Vec<Vec<Field>> = generate_game(
        &mut world,
        &mut buttons,
        total_mine_count,
        area_size,
        controls_panel_size as i32,
        &mine_display
    )?;
    let mut now = SystemTime::now();
    'game: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    break 'game;
                }
                Event::MouseMotion { x, y, .. } => {
                    *world.write_resource::<ui::MouseData>() = ui::MouseData { x, y };
                }
                Event::MouseButtonDown {
                    x, y, mouse_btn, ..
                } => {
                    if let Some(face) = ui::get_overlapping_component_with_type::<FaceButton>(
                        Vector2::new(x, y),
                        world.system_data(),
                    ) {
                        //restart the game
                        grid = generate_game(
                            &mut world,
                            &mut buttons,
                            total_mine_count,
                            area_size,
                            controls_panel_size as i32,
                            &mine_display
                        )?;
                        time = 0;
                        continue;
                    }
                    //we have to offset y due to the fact that controls are on top
                    let y = y - controls_panel_size as i32;
                    match mouse_btn {
                        sdl2::mouse::MouseButton::Left => {
                            //x and y are swapped because i accidentally swapped them in memory
                            reveal_block(
                                Vector2::new(y / 50, x / 50),
                                &mut grid,
                                &mut world,
                                &buttons,
                            );
                            if grid[(y / 50) as usize][(x / 50) as usize].bomb {
                                end_game(false, &mut world, &face, &buttons, &grid, area_size);
                            }
                        }
                        sdl2::mouse::MouseButton::Right => {
                            //this is where we have to put flag on top of the thing
                            if flag_block(
                                Vector2::new(y / 50, x / 50),
                                &mut grid,
                                &mut world,
                                &buttons,
                                &mut flag_count,
                                total_mine_count as i32,
                            ) {
                                update_segmented_display(
                                    &mut world,
                                    &mine_display,
                                    (total_mine_count as i32 - flag_count) as u32,
                                );
                            }
                            if check_mines(&mut grid, area_size) {
                                end_game(true, &mut world, &face, &buttons, &grid, area_size);
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        dispatcher.dispatch(&world);
        render_game(&world, &mut canvas, &texture_manager, &mut game, &font)?;
        //lock frames to run at 30 fps
        //this is minesweeper, why would you want more?
        ::std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 30));
        if let Ok(dur) = now.elapsed() {
            if dur.as_secs_f32() >= 1.0 {
                time += 1;
                now = SystemTime::now();
            }
            update_segmented_display(&mut world, &time_display, time as u32);
        }
    }
    Ok(())
}
