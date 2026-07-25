use crate::ecs::{GridLocation, Player};
use bevy::audio::Volume;
use bevy::prelude::*;

/// max distance from the player that sfx play
pub const SFX_HEARING_RADIUS: f32 = 7.0;

#[derive(Clone, Copy, Debug)]
pub enum Sfx {
    Roll,
    Slide,
    Turn,
    Conveyor,
    PressurePlate,
    Gate,
    BridgeCollapse,
    Success,
}

#[derive(Message, Clone, Copy, Debug)]
pub struct PlaySfx {
    pub sfx: Sfx,
    pub position: Vec3,
}

impl PlaySfx {
    pub fn at(sfx: Sfx, location: &GridLocation) -> Self {
        Self {
            sfx,
            position: location.to_world_space(),
        }
    }

    pub fn at_position(sfx: Sfx, position: Vec3) -> Self {
        Self { sfx, position }
    }
}

#[derive(SystemSet, Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SfxSystems {
    Trigger,
    Playback,
}

#[derive(Resource)]
struct SfxAssets {
    roll: Handle<AudioSource>,
    slide: Handle<AudioSource>,
    turn: Handle<AudioSource>,
    conveyor: Handle<AudioSource>,
    pressure_plate: Handle<AudioSource>,
    gate: Handle<AudioSource>,
    bridge_collapse: Handle<AudioSource>,
    success: Handle<AudioSource>,
}

impl SfxAssets {
    fn get(&self, sfx: Sfx) -> Handle<AudioSource> {
        match sfx {
            Sfx::Roll => self.roll.clone(),
            Sfx::Slide => self.slide.clone(),
            Sfx::Turn => self.turn.clone(),
            Sfx::Conveyor => self.conveyor.clone(),
            Sfx::PressurePlate => self.pressure_plate.clone(),
            Sfx::Gate => self.gate.clone(),
            Sfx::BridgeCollapse => self.bridge_collapse.clone(),
            Sfx::Success => self.success.clone(),
        }
    }
}

pub fn sfx_plugin(app: &mut App) {
    app.add_message::<PlaySfx>()
        .configure_sets(Update, (SfxSystems::Trigger, SfxSystems::Playback).chain())
        .add_systems(Startup, load_sfx)
        .add_systems(Update, play_sfx.in_set(SfxSystems::Playback));
}

fn load_sfx(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(SfxAssets {
        roll: asset_server.load("sfx/terrain/Terrain B.ogg"),
        slide: asset_server.load("sfx/terrain/Terrain F.ogg"),
        turn: asset_server.load("sfx/terrain/Terrain E.ogg"),
        conveyor: asset_server.load("sfx/Conveyer.ogg"),
        pressure_plate: asset_server.load("sfx/Pressure Plate.ogg"),
        gate: asset_server.load("sfx/door/Door A.ogg"),
        bridge_collapse: asset_server.load("sfx/Bridge Collapse.ogg"),
        success: asset_server.load("sfx/success.ogg"),
    });
}

fn play_sfx(
    mut commands: Commands,
    assets: Res<SfxAssets>,
    player: Single<&GridLocation, With<Player>>,
    mut requested_sfx: MessageReader<PlaySfx>,
) {
    let listener_position = player.to_world_space();
    for request in requested_sfx.read() {
        let distance = listener_position.distance(request.position);
        if distance >= SFX_HEARING_RADIUS {
            continue;
        }

        let gain = 1.0 - distance / SFX_HEARING_RADIUS;
        commands.spawn((
            AudioPlayer::new(assets.get(request.sfx)),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(gain)),
        ));
    }
}
