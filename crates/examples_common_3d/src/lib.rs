use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use bevy_xpbd_3d::{XpbdLoop, XpbdPlugin};

#[derive(Default)]
pub struct XpbdExamplePlugin;

impl Plugin for XpbdExamplePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(XpbdPlugin)
            //.add_plugin(bevy_editor_pls::EditorPlugin)
            //.add_plugin(WorldInspectorPlugin)
            .add_plugin(FrameTimeDiagnosticsPlugin)
            .add_state::<AppState>();
        //.add_system(SystemSet::on_enter(AppState::Paused).with_system(bevy_xpbd_3d::pause))
        //.add_system(SystemSet::on_exit(AppState::Paused).with_system(bevy_xpbd_3d::resume))
        //.add_system(pause_button)
        //.add_system(SystemSet::on_update(AppState::Paused).with_system(step_button));
    }
}

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    Paused,
    #[default]
    Running,
}

fn pause_button(
    current: Res<State<AppState>>,
    mut next: ResMut<NextState<AppState>>,
    keys: Res<Input<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::P) {
        next.set(match current.0 {
            AppState::Paused => AppState::Running,
            AppState::Running => AppState::Paused,
        });
    }
}

fn step_button(mut xpbd_loop: ResMut<XpbdLoop>, keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::Return) {
        xpbd_loop.step();
    }
}
