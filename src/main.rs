use game_oxide_framework::*;
use game_oxide_framework::{components::*, game::Game, render::*, texture_manager::*};
use nalgebra::Vector2;
use sdl2::event::Event;
use specs::{
    Builder, Component, Dispatcher, DispatcherBuilder, Entity, EntityBuilder, NullStorage, Read,
    ReadStorage, System, VecStorage, World, WorldExt,
};

use rand::distributions::{Distribution, Uniform};

///Defines the parent of the drop down menu items. this is the thing that gets unwrapped
#[derive(Clone, Debug, PartialEq, Component, Default)]
#[storage(VecStorage)]
pub struct Tile {
    pub position: Vector2<usize>,
    pub value: i32,
    pub is_bomb: bool,
    pub revealed: bool,
}

///Render everything to the screen
pub fn render_game(
    world: &World,
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    textures: &TextureManager,
    game: &mut Game,
    font: &sdl2::ttf::Font,
) -> Result<(), String> {
    canvas.set_draw_color(sdl2::pixels::Color::RGBA(0, 0, 0, 255));
    canvas.clear();
    render_fill(canvas, world.system_data(), game)?;
    render_textures(canvas, textures, world.system_data(), game)?;
    render_text(
        canvas,
        font,
        &canvas.texture_creator(),
        world.system_data(),
        game,
    )?;
    canvas.present();
    Ok(())
}

#[derive(Default, Clone)]
struct Field {
    value: i32,
    bomb: bool,
    border: bool,
    revealed: bool,
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
            if point.x > area_size || point.y > area_size {
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

fn main() -> Result<(), String> {
    let area_size: usize = 10;
    let (mut world, sdl, video_subsystem, ttf_context, mut canvas, mut game) = setup::setup(
        "Rust Minesweeper".to_owned(),
        Some(Vector2::new(area_size as u32 * 50, area_size as u32 * 50)),
    )?;
    let mut dispatcher = DispatcherBuilder::new()
        .with(ui::ButtonUpdateSystem, "button_update_system", &[])
        .build();
    ui::register_ui_components(&mut world);
    let mut event_pump = sdl.event_pump().unwrap();

    let mut texture_creator = canvas.texture_creator();
    let mut texture_manager = TextureManager::new(&texture_creator)?;

    world.register::<Tile>();
    world.insert(ui::MouseData::default());
    let font = ttf_context
        .load_font("./assets/fonts/Roboto-Medium.ttf", 22)
        .unwrap();
    let mut grid = generate_grid(10, 10);
    let mut buttons: Vec<Vec<Entity>> = Vec::new();
    //generate button entities
    for i in 0..area_size {
        buttons.push(Vec::new());
        for j in 0..area_size {
            buttons[i].push(
                ui::make_button_base(
                    &mut world,
                    Vector2::new(j as i32 * 50 + 2, i as i32 * 50 + 2),
                    Vector2::new(45, 45),
                    Some(ui::Button {
                        hovered_over: false,
                        hovered_over_texture_name: None,
                        hovered_over_text: None,
                        hovered_over_color: Some(sdl2::pixels::Color::RGBA(255, 0, 255, 120)),
                        normal_texture_name: None,
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
                .with(Text {
                    text: if grid[i][j].bomb {
                        "B".to_owned()
                    } else {
                        grid[i][j].value.to_string()
                    },
                    color: sdl2::pixels::Color::BLACK,
                    visible: grid[i][j].bomb,
                    offset: Vector2::new(20, 15),
                })
                .with(Tile {
                    position: Vector2::new(i, j),
                    value: grid[i][j].value,
                    is_bomb: grid[i][j].bomb,
                    revealed: false,
                })
                .build(),
            );
        }
    }

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
                    //x and y are swapped because i accidentally swapped them in memory
                    reveal_block(
                        Vector2::new(y / 50, x / 50),
                        &mut grid,
                        &mut world,
                        &buttons,
                    );
                }
                _ => {}
            }
        }
        dispatcher.dispatch(&world);
        render_game(&world, &mut canvas, &texture_manager, &mut game, &font)?;
        //lock frames to run at 30 fps
        //this is minesweeper why would you want more?
        ::std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 30));
    }
    Ok(())
}
