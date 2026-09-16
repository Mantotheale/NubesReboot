use std::fs::File;
use std::io::{BufRead, BufReader};
use crate::constants;
use crate::graphics::texture::texture_atlas::TextureAtlas;
use crate::graphics::texture::texture_handle::TextureHandle;
use crate::math::point2f::Point2f;
use crate::math::positive_f32::PositiveF32;
use crate::math::rect2f::Rect2f;

pub trait App {
    fn init() -> Self;
    fn process_inputs(&mut self);
    fn update(&mut self);
    fn one_sec_update(&mut self);
    fn render(&mut self);
}

struct Map {
    tiles: Vec<(Rect2f, u32, TextureHandle)>
}

pub struct Game {
    //map: Map
}

impl App for Game {
    fn init() -> Self {
        /*let map_file = constants::RESOURCE_DIR.get_file("maps/map.txt").unwrap();
        let reader = BufReader::new(map_file.contents());
        let mut tiles = Vec::new();

        let tiles_path = "tiles/".to_owned();
        for line in reader.lines() {
            let Ok(line) = line else { break; };
            let mut elements = line.split(" ");
            let x: f32 = elements.next().unwrap().parse().unwrap();
            let y: f32 = elements.next().unwrap().parse().unwrap();
            let z_index: u32 = elements.next().unwrap().parse().unwrap();
            let width: f32 = elements.next().unwrap().parse().unwrap();
            let height: f32 = elements.next().unwrap().parse().unwrap();
            let tile_name = elements.next().unwrap();

            let rect = Rect2f::new(
                Point2f::new(x, y),
                PositiveF32::new(width).unwrap(),
                PositiveF32::new(height).unwrap()
            );

            let path = tiles_path.clone() + tile_name;
            println!("{}", path);
            let tile = texture_atlas.get_tile(path.as_ref()).unwrap();
            tiles.push((rect, z_index, tile.clone()));
        }*/

        //Self { map: Map { tiles } }
        Self {}
    }

    fn process_inputs(&mut self) {
    }

    fn update(&mut self) {
    }

    fn one_sec_update(&mut self) {
    }

    fn render(&mut self) {
    }
}