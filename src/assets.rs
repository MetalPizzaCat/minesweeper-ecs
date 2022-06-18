/**This file  contains function that loads all of the textures used by the game=
*/
use nalgebra::Vector4;

///Loads all of the textures that are used by the game
pub fn load_textures(
    texture_manager: &mut game_oxide_framework::texture_manager::TextureManager,
) -> Result<(), String> {
    texture_manager.load(
        Vector4::new(0, 0, 16, 16),
        "tile_default".to_owned(),
        "./assets/minesweeper.png".to_owned(),
    )?;
    texture_manager.load(
        Vector4::new(0, 16, 16, 16),
        "tile_selected".to_owned(),
        "./assets/minesweeper.png".to_owned(),
    )?;
    texture_manager.load(
        Vector4::new(0, 48, 16, 16),
        "tile_question".to_owned(),
        "./assets/minesweeper.png".to_owned(),
    )?;
    texture_manager.load(
        Vector4::new(0, 64, 16, 16),
        "tile_bomb".to_owned(),
        "./assets/minesweeper.png".to_owned(),
    )?;
    texture_manager.load(
        Vector4::new(0, 32, 16, 16),
        "tile_flag".to_owned(),
        "./assets/minesweeper.png".to_owned(),
    )?;
    texture_manager.load(
        Vector4::new(0, 0, 20, 20),
        "face_default".to_owned(),
        "./assets/face.png".to_owned(),
    )?;
    texture_manager.load(
        Vector4::new(20, 0, 20, 20),
        "face_loose".to_owned(),
        "./assets/face.png".to_owned(),
    )?;
    texture_manager.load(
        Vector4::new(40, 0, 20, 20),
        "face_win".to_owned(),
        "./assets/face.png".to_owned(),
    )?;
    texture_manager.load(
        Vector4::new(60, 0, 20, 20),
        "face_hover".to_owned(),
        "./assets/face.png".to_owned(),
    )?;
    //for some reason in the original file for minesweeper it went from 8 to 0 (top to bottom),
    // i decided to keep it that way so we have to do a bit of a weird loop
    //can you even get an 8?
    for i in (0..=8).rev() {
        texture_manager.load(
            Vector4::new(0, i * 16 + 128, 16, 16),
            "tile_".to_owned() + (8 - i).to_string().as_str(),
            "./assets/minesweeper.png".to_owned(),
        )?;
    }
    for i in (0..=9) {
        texture_manager.load(
            Vector4::new(i * 20, 0, 20, 36),
            i.to_string(),
            "./assets/numbers.png".to_owned(),
        )?;
    }
    Ok(())
}
