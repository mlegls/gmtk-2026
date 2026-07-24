use bevy::prelude::Resource;
use eyre::eyre;
use image::RgbaImage;

pub const MAP_WIDTH: usize = 32;
pub const MAP_HEIGHT: usize = 32;
pub type MapLayer = [[u32; MAP_HEIGHT]; MAP_WIDTH];

#[derive(Resource)]
pub struct WorldMap {
    pub ground: MapLayer,
    pub stuff: MapLayer,
}

pub fn load_world_map() -> Result<WorldMap, eyre::Error> {
    Ok(WorldMap {
        ground: load_layer(include_bytes!("../assets/maps/ground.png"), "ground")?,
        stuff: load_layer(include_bytes!("../assets/maps/stuff.png"), "stuff")?,
    })
}

fn load_layer(bytes: &[u8], layer: &'static str) -> Result<MapLayer, eyre::Error> {
    let image = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)?
        .into_rgba8();

    check_dimensions(&image, layer)?;

    let mut output = [[0; MAP_HEIGHT]; MAP_WIDTH];
    for (x, y, pixel) in image.enumerate_pixels() {
        output[x as usize][y as usize] = match pixel.0 {
            [_, _, _, 0] => 0, // void
            [255, 255, 255, 255] => 1, // ground
            [154, 114, 46, 255] => 3, // bridge
            rgba => {
                return Err(eyre!("invalid pixel: {x} {y} {rgba:?}"))
            }
        };
    }
    Ok(output)
}

fn check_dimensions(image: &RgbaImage, layer: &'static str) -> Result<(), eyre::Error> {
    let (width, height) = image.dimensions();
    if width != MAP_WIDTH as u32 || height != MAP_HEIGHT as u32 {
        return Err(eyre!("wrong size: {layer} (should be {MAP_WIDTH} {MAP_HEIGHT}; got {width} {height})"));
    }
    Ok(())
}
