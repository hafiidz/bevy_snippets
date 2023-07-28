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
        // ==== Game logic ==== //
        .add_systems(Startup, (setup_camera, load))
        .add_systems(Update, spawn.run_if(prototype_ready(PROTO_ID)))
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
