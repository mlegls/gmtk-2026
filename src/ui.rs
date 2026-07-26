use crate::ecs::{
    AvailableActions, CompletedTurn, Player, PlayerAction, TurnCountText, TurnCounter,
};
use crate::story::ResetGame;
use bevy::prelude::*;

const RESET_CONFIRMATION: &str = "are you sure you'd like to reset? press r again if so.";
const ADD_TURNS_CONFIRMATION: &str =
    "are you sure you'd like to add 10 turns? press f again if so.";
const TURN_COUNT_SUFFIX: &str = "turns until the end of the world";

pub(crate) fn turn_count_label(turns: u32) -> String {
    format!("{turns} {TURN_COUNT_SUFFIX}")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UtilityAction {
    Reset,
    AddTurns,
}

#[derive(Resource, Default)]
pub(crate) struct UtilityConfirmation(Option<UtilityAction>);

#[derive(Component)]
pub(crate) struct UtilityConfirmationText;

#[derive(Component)]
struct ActionKey(PlayerAction);

const ACTION_GRID: [Option<(PlayerAction, &str)>; 12] = [
    Some((PlayerAction::TurnLeft, "Q")),
    Some((PlayerAction::RollForward, "W")),
    Some((PlayerAction::TurnRight, "E")),
    Some((PlayerAction::RollLeft, "A")),
    Some((PlayerAction::RollBackward, "S")),
    Some((PlayerAction::RollRight, "D")),
    Some((PlayerAction::SlideLeft, "Z")),
    Some((PlayerAction::TurnAround, "X")),
    Some((PlayerAction::SlideRight, "C")),
    None,
    Some((PlayerAction::Wait, "_")),
    None,
];

pub fn ui_plugin(app: &mut App) {
    app.init_resource::<UtilityConfirmation>()
        .add_systems(Startup, (setup_utility_confirmation, setup_action_grid))
        .add_systems(
            Update,
            (update_turn_count.after(utility_buttons), update_action_grid),
        );
}

pub fn ui() -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            height: percent(100),
        }
        Children [
            (
                template(|ctx| {
                    Ok(Text(turn_count_label(ctx.resource::<TurnCounter>().0)))
                })
                TextFont {
                    font_size: px(24.0),
                }
                Node {
                    position_type: PositionType::Absolute,
                    top: px(24.0),
                    left: px(24.0),
                }
                TurnCountText
            )
        ]
    }
}

fn setup_action_grid(mut commands: Commands) {
    commands
        .spawn((
            Name::new("available actions"),
            Node {
                position_type: PositionType::Absolute,
                top: px(64.0),
                left: px(16.0),
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::px(3, 52.0),
                grid_template_rows: RepeatedGridTrack::px(4, 44.0),
                row_gap: px(6.0),
                column_gap: px(6.0),
                padding: UiRect::all(px(8.0)),
                ..default()
            },
            GlobalZIndex(10),
        ))
        .with_children(|grid| {
            for cell in ACTION_GRID {
                let Some((action, key)) = cell else {
                    grid.spawn(Node::default());
                    continue;
                };
                grid.spawn((
                    Name::new(format!("{key} action")),
                    Text::new(key),
                    TextFont::from_font_size(22.0),
                    TextColor(Color::WHITE),
                    TextLayout::justify(Justify::Center),
                    Node {
                        width: percent(100.0),
                        height: percent(100.0),
                        padding: UiRect::vertical(px(8.0)),
                        ..default()
                    },
                    ActionKey(action),
                ));
            }
        });
}

fn update_action_grid(
    available_actions: Single<&AvailableActions, With<Player>>,
    mut action_keys: Query<(&ActionKey, &mut TextColor)>,
) {
    for (action_key, mut text_color) in &mut action_keys {
        text_color.0 = if available_actions.contains(action_key.0) {
            Color::WHITE
        } else {
            Color::srgba(0.65, 0.65, 0.65, 0.3)
        };
    }
}

fn setup_utility_confirmation(mut commands: Commands) {
    commands.spawn((
        Name::new("utility confirmation"),
        Text::default(),
        TextFont::from_font_size(24.0),
        TextColor(Color::WHITE),
        TextLayout::justify(Justify::Center),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(64.0),
            width: percent(100.0),
            padding: UiRect::all(px(12.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        GlobalZIndex(20),
        Visibility::Hidden,
        UtilityConfirmationText,
    ));
}

pub(crate) fn utility_buttons(
    keys: Res<ButtonInput<KeyCode>>,
    mut confirmation: ResMut<UtilityConfirmation>,
    mut confirmation_text: Single<(&mut Text, &mut Visibility), With<UtilityConfirmationText>>,
    mut turn_counter: ResMut<TurnCounter>,
    mut resets: MessageWriter<ResetGame>,
) {
    if let Some(pending) = confirmation.0 {
        let confirmation_key = match pending {
            UtilityAction::Reset => KeyCode::KeyR,
            UtilityAction::AddTurns => KeyCode::KeyF,
        };
        let confirmed = keys.just_pressed(confirmation_key);
        let other_key_pressed = keys.get_just_pressed().any(|key| *key != confirmation_key);

        if confirmed && !other_key_pressed {
            match pending {
                UtilityAction::Reset => {
                    resets.write(ResetGame);
                }
                UtilityAction::AddTurns => turn_counter.0 = turn_counter.0.saturating_add(10),
            }
        }
        if confirmed || other_key_pressed {
            confirmation.0 = None;
            *confirmation_text.1 = Visibility::Hidden;
        }
        return;
    }

    let requested = if keys.just_pressed(KeyCode::KeyR) {
        UtilityAction::Reset
    } else if keys.just_pressed(KeyCode::KeyF) {
        UtilityAction::AddTurns
    } else {
        return;
    };

    confirmation.0 = Some(requested);
    **confirmation_text.0 = match requested {
        UtilityAction::Reset => RESET_CONFIRMATION,
        UtilityAction::AddTurns => ADD_TURNS_CONFIRMATION,
    }
    .into();
    *confirmation_text.1 = Visibility::Visible;
}

fn update_turn_count(
    turn_counter: Res<TurnCounter>,
    mut turn_count_text: Single<&mut Text, With<TurnCountText>>,
    mut completed_turn_recv: MessageReader<CompletedTurn>,
) {
    let completed_turn = completed_turn_recv.read().next().is_some();
    if completed_turn || turn_counter.is_changed() {
        ***turn_count_text = turn_count_label(turn_counter.0);
    }
}
