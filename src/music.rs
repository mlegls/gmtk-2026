use crate::ecs::{AvailableActions, Player, PlayerAction};
use bevy::audio::{AudioSinkPlayback, Volume};
use bevy::prelude::*;

const STEMS: [(PlayerAction, &str); 10] = [
    (PlayerAction::RollForward, "music/bgm/Flute_1.ogg"),
    (PlayerAction::RollBackward, "music/bgm/Synth_1.ogg"),
    (PlayerAction::RollLeft, "music/bgm/Organ_1.ogg"),
    (PlayerAction::RollRight, "music/bgm/Piano_1.ogg"),
    (PlayerAction::TurnLeft, "music/bgm/Strings_1.ogg"),
    (PlayerAction::TurnRight, "music/bgm/Wurly_1.ogg"),
    (PlayerAction::TurnAround, "music/bgm/Triangle_1.ogg"),
    (PlayerAction::SlideLeft, "music/bgm/Vox Hi_1.ogg"),
    (PlayerAction::SlideRight, "music/bgm/Vox Lo_1.ogg"),
    (PlayerAction::Wait, "music/bgm/Drumkit_1.ogg"),
];

#[derive(Resource)]
struct BgmAssets(Vec<(PlayerAction, Handle<AudioSource>)>);

#[derive(Component)]
struct BgmStem {
    action: PlayerAction,
    enabled: bool,
}

pub fn music_plugin(app: &mut App) {
    app.add_systems(Startup, load_bgm)
        .add_systems(Update, (start_bgm, update_bgm_layers));
}

fn load_bgm(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(BgmAssets(
        STEMS
            .iter()
            .map(|(action, path)| (*action, asset_server.load(*path)))
            .collect(),
    ));
}

fn start_bgm(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    assets: Res<BgmAssets>,
    available_actions: Single<&AvailableActions, With<Player>>,
    active_stems: Query<(), With<BgmStem>>,
) {
    // wait for stems to load
    if !active_stems.is_empty()
        || !assets
            .0
            .iter()
            .all(|(_, handle)| asset_server.is_loaded_with_dependencies(handle))
    {
        return;
    }

    // spawn stems
    for (action, source) in &assets.0 {
        let enabled = available_actions.contains(*action);
        commands.spawn((
            AudioPlayer::new(source.clone()),
            PlaybackSettings::LOOP.with_volume(if enabled {
                Volume::Linear(1.0)
            } else {
                Volume::SILENT
            }),
            BgmStem {
                action: *action,
                enabled,
            },
        ));
    }
}

/// mute stems based on disabled skills
fn update_bgm_layers(
    available_actions: Single<&AvailableActions, With<Player>>,
    mut stems: Query<(&mut BgmStem, &mut AudioSink)>,
) {
    for (mut stem, mut sink) in &mut stems {
        let enabled = available_actions.contains(stem.action);
        if enabled != stem.enabled {
            sink.set_volume(if enabled {
                Volume::Linear(1.0)
            } else {
                Volume::SILENT
            });
            stem.enabled = enabled;
        }
    }
}
