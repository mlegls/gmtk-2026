use crate::bridge::Bridge;
use crate::ecs::{
    Altar, Arrow, ArrowBlock, AvailableActions, CameraRig, Direction, Gate, GridLocation,
    InitialObstructedSet, Moving, ObstructedSet, Orientation, Player, PlayerAction, SignalSystems,
    TurnCountText, TurnCounter,
};
use crate::movement::CameraTurn;
use crate::signal_logic::{SwitchStates, TimerBank};
use crate::{MAX_TURN_COUNT, PLAYER_START};
use bevy::prelude::*;
use bevy::text::LineBreak;
use std::collections::VecDeque;

const STORY_TURNS: u8 = 3;

const BEGINNING: [&str; 2] = [
    "unfortunately, the world is ending.",
    "there's nothing you can do about it. so might as well solve some puzzles.",
];

const SKILL_STORIES: [&str; 8] = [
    "did you think you would get stronger?\nsorry. there's really nothing you can do.\n when the countdown hits 0, everything is gone.",
    "another skill lost. are things getting harder?\ndon't worry, you'll get used to it. hopefully.",
    "isn't it kind of ironic how you get to choose what to lose?\nis that what they call the \"illusion of choice\", like when you offer a baby broccoli or carrots?\nisn't it all the same in the end, in any case?",
    "with this much lost, you'll have to work pretty hard to compensate.\ngood thing a lot of your skills are redundant.\n why'd you spend the time learning all that...",
    "have you seen everything yet? there's a lot out there.\ni've heard there's a place you can see the whole world at once.\ntoo bad the world is ending so soon. maybe next time.",
    "you can still move, right? you should be careful about that.\nwell, maybe it doesn't matter, since it's all gonna be over soon anyway.",
    "man, it kinda feels like you're doing the same things over and over at this point, doesn't it?\nthere's almost something beautiful in how you can still get anywhere at all.\nlike a little worm, flailing around with its head chopped off.",
    "almost there, i guess. nothing much left to say at this point.",
];

const NORMAL_ENDING: &str = "it's over... right? it's kind of anticlimactic, isn't it...";
const TRUE_NINTH: &str = "huh... isn't this where it usually ends?";
const TRUE_ENDING: &str =
    "congrats, you did it. you lasted until the very end. now it's really over.";

const TRUE_ORDER: [PlayerAction; 10] = [
    PlayerAction::Wait,
    PlayerAction::TurnAround,
    PlayerAction::SlideLeft,
    PlayerAction::SlideRight,
    PlayerAction::RollBackward,
    PlayerAction::RollLeft,
    PlayerAction::RollRight,
    PlayerAction::TurnLeft,
    PlayerAction::TurnRight,
    PlayerAction::RollForward,
];

#[derive(States, Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum GamePhase {
    #[default]
    Playing,
    Ending,
}

#[derive(Message, Clone, Copy, Debug)]
pub struct SkillSacrificed(pub PlayerAction);

#[derive(Message, Clone, Copy, Debug)]
pub(crate) struct ResetGame;

#[derive(Resource, Default)]
struct SacrificeHistory(Vec<PlayerAction>);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StoryBeat {
    Skill(usize),
    TrueNinth,
    NormalEnding,
    TrueEnding,
}

fn story_beat(history: &[PlayerAction]) -> Option<StoryBeat> {
    match history.len() {
        1..=8 => Some(StoryBeat::Skill(history.len() - 1)),
        9 if history == &TRUE_ORDER[..9] => Some(StoryBeat::TrueNinth),
        9 => Some(StoryBeat::NormalEnding),
        10 if history == TRUE_ORDER => Some(StoryBeat::TrueEnding),
        _ => None,
    }
}

#[derive(Resource)]
struct Narration {
    queue: VecDeque<&'static str>,
    active: bool,
}

impl Default for Narration {
    fn default() -> Self {
        Self {
            queue: beginning_queue(),
            active: false,
        }
    }
}

fn beginning_queue() -> VecDeque<&'static str> {
    let mut queue = VecDeque::new();
    for message in BEGINNING {
        queue_message(&mut queue, message);
    }
    queue
}

fn queue_message(queue: &mut VecDeque<&'static str>, message: &'static str) {
    queue.extend(message.lines());
}

#[derive(Component)]
struct StoryText {
    turns_remaining: u8,
}

#[derive(SystemSet, Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum StorySystems {
    Reset,
    Events,
    Display,
}

pub fn story_plugin(app: &mut App) {
    app.init_state::<GamePhase>()
        .init_resource::<SacrificeHistory>()
        .init_resource::<Narration>()
        .add_message::<SkillSacrificed>()
        .add_message::<ResetGame>()
        .configure_sets(
            Update,
            (
                StorySystems::Reset.before(SignalSystems::Write),
                StorySystems::Events,
                StorySystems::Display,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (request_reset, reset_player, reset_level, reset_timers)
                .chain()
                .in_set(StorySystems::Reset),
        )
        .add_systems(Update, handle_sacrifice.in_set(StorySystems::Events))
        .add_systems(Update, display_story.in_set(StorySystems::Display));
}

fn handle_sacrifice(
    mut sacrifices: MessageReader<SkillSacrificed>,
    mut history: ResMut<SacrificeHistory>,
    mut narration: ResMut<Narration>,
    mut next_phase: ResMut<NextState<GamePhase>>,
) {
    for sacrifice in sacrifices.read() {
        history.0.push(sacrifice.0);
        match story_beat(&history.0) {
            Some(StoryBeat::Skill(index)) => {
                queue_message(&mut narration.queue, SKILL_STORIES[index]);
            }
            Some(StoryBeat::TrueNinth) => queue_message(&mut narration.queue, TRUE_NINTH),
            Some(StoryBeat::NormalEnding) => {
                queue_message(&mut narration.queue, NORMAL_ENDING);
                next_phase.set(GamePhase::Ending);
            }
            Some(StoryBeat::TrueEnding) => {
                queue_message(&mut narration.queue, TRUE_ENDING);
                next_phase.set(GamePhase::Ending);
            }
            None => {}
        }
    }
}

fn request_reset(
    phase: Res<State<GamePhase>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut reset: MessageWriter<ResetGame>,
    mut history: ResMut<SacrificeHistory>,
    mut narration: ResMut<Narration>,
    story_text: Query<Entity, With<StoryText>>,
    mut commands: Commands,
    mut next_phase: ResMut<NextState<GamePhase>>,
) {
    if *phase.get() != GamePhase::Ending
        || !PlayerAction::ALL
            .iter()
            .any(|action| keys.just_pressed(action.key_code()))
    {
        return;
    }

    history.0.clear();
    narration.queue = beginning_queue();
    narration.active = false;
    for entity in &story_text {
        commands.entity(entity).despawn();
    }
    reset.write(ResetGame);
    next_phase.set(GamePhase::Playing);
}

fn reset_player(
    mut reset: MessageReader<ResetGame>,
    player: Single<
        (
            Entity,
            &mut Transform,
            &mut GridLocation,
            &mut Orientation,
            &mut AvailableActions,
        ),
        With<Player>,
    >,
    mut arrow: Single<&mut Transform, (With<Arrow>, Without<Player>)>,
    camera: Single<Entity, With<CameraRig>>,
    mut turn_counter: ResMut<TurnCounter>,
    mut turn_count_text: Single<&mut Text, With<TurnCountText>>,
    mut commands: Commands,
) {
    if reset.read().next().is_none() {
        return;
    }

    let (entity, mut transform, mut location, mut orientation, mut actions) = player.into_inner();
    location.0 = PLAYER_START;
    transform.translation = location.to_world_space() + vec3(0.0, 0.5, 0.0);
    transform.rotation = Quat::IDENTITY;
    *orientation = Orientation(Direction::North);
    *actions = AvailableActions::default();
    arrow.rotation = Quat::IDENTITY;
    **turn_counter = MAX_TURN_COUNT;
    ***turn_count_text = MAX_TURN_COUNT.to_string();
    commands.entity(entity).remove::<Moving>();
    commands.entity(*camera).remove::<CameraTurn>();
}

fn reset_level(
    mut reset: MessageReader<ResetGame>,
    initial_obstructions: Res<InitialObstructedSet>,
    mut obstructed_set: ResMut<ObstructedSet>,
    mut altars: Query<
        (&GridLocation, &mut Transform),
        (With<Altar>, Without<Gate>, Without<ArrowBlock>),
    >,
    mut gates: Query<
        (&mut Gate, &mut Transform),
        (With<Gate>, Without<Altar>, Without<ArrowBlock>),
    >,
    mut arrow_blocks: Query<
        (&ArrowBlock, &mut Orientation, &mut Transform),
        (Without<Altar>, Without<Gate>),
    >,
    mut bridges: Query<
        (&mut Bridge, &mut Transform),
        (Without<Altar>, Without<Gate>, Without<ArrowBlock>),
    >,
) {
    if reset.read().next().is_none() {
        return;
    }

    obstructed_set.0.clone_from(&initial_obstructions.0);
    for (location, mut transform) in &mut altars {
        transform.translation = location.to_world_space() + vec3(0.0, -5.0, 0.0);
    }
    for (mut gate, mut transform) in &mut gates {
        *gate = Gate::Closed;
        transform.translation.y = 0.0;
    }
    for (arrow, mut orientation, mut transform) in &mut arrow_blocks {
        orientation.0 = arrow.initial_orientation;
        transform.rotation = arrow.initial_rotation;
    }
    for (mut bridge, mut transform) in &mut bridges {
        bridge.reset(&mut transform);
    }
}

fn reset_timers(
    mut reset: MessageReader<ResetGame>,
    mut switches: ResMut<SwitchStates>,
    mut timers: Query<&mut TimerBank>,
) {
    if reset.read().next().is_none() {
        return;
    }

    switches.reset();
    for mut timer in &mut timers {
        timer.reset();
    }
}

fn display_story(
    mut narration: ResMut<Narration>,
    mut completed_turns: MessageReader<crate::ecs::CompletedTurn>,
    player: Single<&Transform, (With<Player>, Without<StoryText>)>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut current_text: Query<(Entity, &mut StoryText, &mut Node, &mut TextColor)>,
    mut commands: Commands,
) {
    let turns_completed = completed_turns.read().count().min(u8::MAX as usize) as u8;
    let (camera, camera_transform) = camera.into_inner();
    let Ok(viewport_position) =
        camera.world_to_viewport(camera_transform, player.translation + vec3(0.0, 2.1, 0.0))
    else {
        return;
    };

    for (entity, mut story, mut node, mut color) in &mut current_text {
        node.left = px(viewport_position.x - 300.0);
        node.top = px(viewport_position.y);

        story.turns_remaining = story.turns_remaining.saturating_sub(turns_completed);
        color
            .0
            .set_alpha(story.turns_remaining as f32 / STORY_TURNS as f32);

        if story.turns_remaining == 0 {
            commands.entity(entity).despawn();
            narration.active = false;
        }
        return;
    }

    if narration.active {
        return;
    }
    let Some(line) = narration.queue.pop_front() else {
        return;
    };

    narration.active = true;
    commands.spawn((
        Name::new("story"),
        Text::new(line),
        TextFont::from_font_size(28.0),
        TextColor(Color::WHITE),
        TextLayout::new(Justify::Center, LineBreak::WordBoundary),
        Node {
            position_type: PositionType::Absolute,
            left: px(viewport_position.x - 300.0),
            top: px(viewport_position.y),
            width: px(600.0),
            ..default()
        },
        GlobalZIndex(10),
        StoryText {
            turns_remaining: STORY_TURNS,
        },
    ));
}
