use crate::ecs::Altar;
use crate::map_json::{InitialTimerSlots, MapJson, ReplaceTimers, SignalExpression, TimerTemplate};
use bevy::math::{UVec2, uvec2};
use bevy::prelude::Resource;
use eyre::eyre;
use image::RgbaImage;
use std::collections::HashMap;

pub const MAP_WIDTH: usize = 55;
pub const MAP_HEIGHT: usize = 105;
pub type MapLayer<T> = [[T; MAP_HEIGHT]; MAP_WIDTH];
pub type TileLayer = MapLayer<MapTile>;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum GroundTile {
    #[default]
    Void,
    Ground,
    Conveyor,
    ArrowBlock,
    Altar,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum StuffTile {
    #[default]
    None,
    PressurePlate,
    Bridge,
    Gate,
    Wall,
    FaceId,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MapTile {
    pub ground: GroundTile,
    pub stuff: StuffTile,
}

#[derive(Resource)]
pub struct WorldMap {
    pub tiles: TileLayer,
    pub orientation: MapLayer<u32>,
    pub touched_switches: HashMap<UVec2, Vec<String>>,
    pub timer_banks: HashMap<UVec2, InitialTimerSlots>,
    pub activation_conditions: HashMap<UVec2, Vec<SignalExpression>>,
    pub input_effects: HashMap<UVec2, Vec<ReplaceTimers>>,
    pub timers: HashMap<String, TimerTemplate>,
    pub altars: HashMap<UVec2, Altar>,
}

pub fn load_world_map(map_json: &MapJson) -> Result<WorldMap, eyre::Error> {
    let mut tile_mapping = HashMap::new();
    tile_mapping.insert(
        [255, 255, 255, 255],
        MapTile {
            ground: GroundTile::Ground,
            stuff: StuffTile::None,
        },
    );
    tile_mapping.insert(
        [168, 73, 255, 255],
        MapTile {
            ground: GroundTile::Altar,
            stuff: StuffTile::None,
        },
    );
    tile_mapping.insert(
        [99, 99, 99, 255],
        MapTile {
            ground: GroundTile::Conveyor,
            stuff: StuffTile::None,
        },
    );
    tile_mapping.insert(
        [0, 255, 0, 255],
        MapTile {
            ground: GroundTile::ArrowBlock,
            stuff: StuffTile::None,
        },
    );
    tile_mapping.insert(
        [255, 0, 0, 255],
        MapTile {
            ground: GroundTile::Ground,
            stuff: StuffTile::PressurePlate,
        },
    );
    tile_mapping.insert(
        [154, 114, 46, 255],
        MapTile {
            ground: GroundTile::Void,
            stuff: StuffTile::Bridge,
        },
    );
    tile_mapping.insert(
        [198, 156, 109, 255],
        MapTile {
            ground: GroundTile::Ground,
            stuff: StuffTile::Bridge,
        },
    );
    tile_mapping.insert(
        [0, 38, 255, 255],
        MapTile {
            ground: GroundTile::Ground,
            stuff: StuffTile::Gate,
        },
    );
    tile_mapping.insert(
        [64, 64, 64, 255],
        MapTile {
            ground: GroundTile::Ground,
            stuff: StuffTile::Wall,
        },
    );
    tile_mapping.insert(
        [4, 255, 26, 255],
        MapTile {
            ground: GroundTile::Ground,
            stuff: StuffTile::FaceId,
        },
    );

    let mut orientation_mapping = HashMap::new();
    orientation_mapping.insert([0, 255, 255, 255], 1); // north
    orientation_mapping.insert([255, 255, 0, 255], 2); // west
    orientation_mapping.insert([255, 0, 255, 255], 3); // south
    orientation_mapping.insert([0, 0, 0, 255], 4); // east

    let mut touched_switches: HashMap<UVec2, Vec<String>> = HashMap::new();
    let mut timer_banks = HashMap::new();
    let mut activation_conditions: HashMap<UVec2, Vec<SignalExpression>> = HashMap::new();
    let mut input_effects: HashMap<UVec2, Vec<ReplaceTimers>> = HashMap::new();
    for association in &map_json.associations {
        let position = association.position.into();

        if let Some(slots) = &association.timers {
            timer_banks.insert(position, slots.clone());
        }

        if let Some(expression) = &association.activated_by {
            activation_conditions
                .entry(position)
                .or_default()
                .push(expression.clone());
        }

        if let Some(effect) = &association.on_activate {
            if let Some(switch) = &effect.touch_switch {
                touched_switches
                    .entry(position)
                    .or_default()
                    .push(switch.clone());
            }

            if let Some(replacement) = &effect.replace_timers {
                let target = replacement.position.into();
                timer_banks
                    .entry(target)
                    .or_insert_with(|| [None, None, None]);
                input_effects
                    .entry(position)
                    .or_default()
                    .push(replacement.clone());
            }
        }
    }

    let mut altars = HashMap::new();
    for altar in map_json.altars.clone() {
        altars.insert(
            uvec2(altar.position[0], altar.position[1]),
            Altar(altar.action),
        );
    }

    let tiles = load_layer(
        //include_bytes!("../assets/maps/map.png"),
        std::fs::read("assets/maps/map.png").unwrap().as_ref(),
        "map",
        tile_mapping,
    )?;
    let orientation = load_layer(
        //include_bytes!("../assets/maps/orientation.png"),
        std::fs::read("assets/maps/orientation.png").unwrap().as_ref(),
        "orientation",
        orientation_mapping,
    )?;

    for (x, column) in tiles.iter().enumerate() {
        for (y, tile) in column.iter().enumerate() {
            if tile.ground == GroundTile::Altar && !altars.contains_key(&uvec2(x as u32, y as u32))
            {
                return Err(eyre!("altar at {x} {y} has no action in map.json"));
            }
        }
    }
    for position in altars.keys() {
        if tiles
            .get(position.x as usize)
            .and_then(|column| column.get(position.y as usize))
            .is_none_or(|tile| tile.ground != GroundTile::Altar)
        {
            return Err(eyre!(
                "altar action at {} {} has no purple tile in map.png",
                position.x,
                position.y
            ));
        }
    }

    Ok(WorldMap {
        tiles,
        orientation,
        touched_switches,
        timer_banks,
        activation_conditions,
        input_effects,
        timers: map_json.timers.clone(),
        altars,
    })
}

fn load_layer<T: Copy + Default>(
    bytes: &[u8],
    layer: &'static str,
    color_mapping: HashMap<[u8; 4], T>,
) -> Result<MapLayer<T>, eyre::Error> {
    let image = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)?.into_rgba8();

    check_dimensions(&image, layer)?;

    let mut output = [[T::default(); MAP_HEIGHT]; MAP_WIDTH];
    for (x, column) in output.iter_mut().enumerate() {
        for (y, tile) in column.iter_mut().enumerate() {
            let pixel = image.get_pixel(x as u32, y as u32);
            *tile = if let Some(mapping) = color_mapping.get(&pixel.0) {
                *mapping
            } else if pixel.0[3] == 0 {
                T::default()
            } else {
                return Err(eyre!("invalid tile pixel: {x} {y} {:?}", pixel.0));
            };
        }
    }
    Ok(output)
}

fn check_dimensions(image: &RgbaImage, layer: &'static str) -> Result<(), eyre::Error> {
    let (width, height) = image.dimensions();
    let expected_width = MAP_WIDTH as u32;
    let expected_height = MAP_HEIGHT as u32;
    if width != expected_width || height != expected_height {
        return Err(eyre!(
            "wrong size: {layer} (should be {expected_width} {expected_height}; got {width} {height})"
        ));
    }
    Ok(())
}
