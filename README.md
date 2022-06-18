# Minesweeper in rust
This is a version of minesweeper written in rust using [Game Oxide Framework](https://github.com/MetalPizzaCat/GameOxideFramework) as the base. 
This project was done simply to test and improve [Game Oxide Framework](https://github.com/MetalPizzaCat/GameOxideFramework)

# Features
Most basic minesweeper gameplay. You can
* Click on tile
* Flag tile
* *explode*
* Click on the face to try again

# Code
There are two code files which only need to exist because of framework used
* Game play code is located in `main.rs`, this is the main file of the project. 
* `assets.rs` is file unrelated to gameplay itself, and is only a way of loading textures
* `minesweeper_ui.rs` is also unrelated to gameplay, it only makes buttons

# Notes
* This code was written from 0 so there could be differences in how gameplay feels compared to the original minesweeper
* Game always runs on 10x10 field with 10 bombs, there are no ways of changing that
* There are probably bugs 
* The art itself is not provided with code, so game will appear to look very purple if you just build it.  For version that contains art see releases

# Some images
![Screenshot_2654](https://user-images.githubusercontent.com/36876492/174420233-96cad8bf-7aeb-475a-a85e-5c5acabef308.png)
![Screenshot_2655](https://user-images.githubusercontent.com/36876492/174420235-45e8744c-9577-4832-a2c0-a989769b69d9.png)
