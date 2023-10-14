use bevy::asset::ChangeWatcher;
use bevy::prelude::*;
use bevy_proto::prelude::*;
use std::time::Duration;

fn main() {
    // Used when debugging
    // std::env::set_var("RUST_BACKTRACE", "full");
    // std::env::set_var("RUST_BACKTRACE", "1");

    App::new()
        // add default bevy plugins with hot-reloading enabled
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                // Enable hot-reloading of assets:
                watch_for_changes: ChangeWatcher::with_delay(Duration::from_millis(200)),
                ..default()
            }),
            ProtoPlugin::new(),
        ))
        // ==== Add our own plugin ==== //
        .register_type::<MenuRoot>()
        .init_resource::<IsMenuRootSpawned>()
        .add_state::<AppState>()
        // ==== Game logic ==== //
        .add_systems(Startup, (setup_camera, load))
        .add_systems(Update, spawn.run_if(prototype_ready(PROTO_ID)))
        .add_systems(Update, button_system)
        .add_systems(Update, transition_app_state)
        .run();
}

#[derive(Component, Reflect)]
pub struct MainCamera;

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MainCamera));
}

const PROTO_ID: &str = "MenuRoot";

#[derive(Component, Schematic, Reflect, Debug)]
#[reflect(Schematic)]
pub struct MenuRoot;

#[derive(Resource)]
pub struct IsMenuRootSpawned {
    pub value: bool,
}

impl Default for IsMenuRootSpawned {
    fn default() -> IsMenuRootSpawned {
        IsMenuRootSpawned { value: false }
    }
}

fn load(mut prototypes: PrototypesMut) {
    prototypes.load("MenuRoot.prototype.ron");
}

fn spawn(
    mut proto_commands: ProtoCommands,
    mut commands: Commands,
    mut previous: Local<Option<Entity>>,
    mut proto_asset_events: EventReader<ProtoAssetEvent>,
    mut is_spawned: ResMut<IsMenuRootSpawned>,
) {
    // when the children has been de-spawned via despawn command, re-spawn
    // else re-spawn also when there is some changes to the assets (i.e. Root.prototype.ron)
    // this allows for spawn and despawn when transitioning from and to Main Menu
    if previous.is_none() || !is_spawned.value {
        *previous = Some(proto_commands.spawn(PROTO_ID).id());
        is_spawned.value = true;
    } else {
        for proto_asset_event in proto_asset_events.iter() {
            if proto_asset_event.is_modified(PROTO_ID) {
                commands.entity(previous.unwrap()).despawn_recursive();
                is_spawned.value = false;
            }
        }
    }
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    Game,
}

fn transition_app_state(
    keyboard_input: Res<Input<KeyCode>>,
    app_state: Res<State<AppState>>,
    mut app_state_next_state: ResMut<NextState<AppState>>,
) {
    if keyboard_input.just_pressed(KeyCode::G) {
        if *app_state.get() != AppState::Game {
            app_state_next_state.set(AppState::Game);
            dbg!("Entered AppState::Game");
        } else {
            dbg!("Already in AppState::Game");
        }
    } else if keyboard_input.just_pressed(KeyCode::M) {
        if *app_state.get() != AppState::MainMenu {
            app_state_next_state.set(AppState::MainMenu);
            dbg!("Entered AppState::MainMenu");
        } else {
            dbg!("Already in AppState::MainMenu");
        }
    }
}

const NORMAL_BUTTON: Color = Color::WHITE;
const HOVERED_BUTTON: Color = Color::GRAY;
const PRESSED_BUTTON: Color = Color::GREEN;

fn button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
    app_state: Res<State<AppState>>,
    mut app_state_next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, mut color, mut border_color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                text.sections[0].value = "Press".to_string();
                *color = PRESSED_BUTTON.into();
                border_color.0 = Color::WHITE;

                // toggle state between Game and MainMenu
                if *app_state.get() == AppState::Game {
                    app_state_next_state.set(AppState::MainMenu);
                    dbg!("Move to MainMenu");
                } else {
                    app_state_next_state.set(AppState::Game);
                    dbg!("Move to Game");
                };
            }
            Interaction::Hovered => {
                text.sections[0].value = "Hover".to_string();
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
            Interaction::None => {
                let value = if *app_state.get() == AppState::Game {
                    "Game"
                } else {
                    "MainMenu"
                };
                // text.sections[0].value = "Button".to_string();
                text.sections[0].value = value.to_owned();
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
        }
    }
}
