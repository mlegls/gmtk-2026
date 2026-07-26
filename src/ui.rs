use crate::ecs::{CompletedTurn, TurnCountText, TurnCounter};
use crate::story::ResetGame;
use bevy::prelude::*;

const RESET_CONFIRMATION: &str = "are you sure you'd like to reset? press r again if so.";
const ADD_TURNS_CONFIRMATION: &str =
    "are you sure you'd like to add 10 turns? press f again if so.";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UtilityAction {
    Reset,
    AddTurns,
}

#[derive(Resource, Default)]
pub(crate) struct UtilityConfirmation(Option<UtilityAction>);

#[derive(Component)]
pub(crate) struct UtilityConfirmationText;

pub fn ui_plugin(app: &mut App) {
    app.init_resource::<UtilityConfirmation>()
        .add_systems(Startup, setup_utility_confirmation)
        .add_systems(Update, update_turn_count.after(utility_buttons));
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
                    Ok(Text(ctx.resource::<TurnCounter>().to_string()))
                })
                TurnCountText
            )
        ]
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
        ***turn_count_text = turn_counter.to_string();
    }
}
