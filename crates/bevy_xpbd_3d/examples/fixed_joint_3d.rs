use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;
use examples_common_3d::XpbdExamplePlugin;

#[derive(Component)]
struct Player;

#[derive(Component, Deref, DerefMut)]
pub struct MoveSpeed(pub f64);

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let cube = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));

    let blue = materials.add(StandardMaterial {
        base_color: Color::rgb(0.2, 0.6, 0.8),
        ..default()
    });

    let anchor = commands
        .spawn(PbrBundle {
            mesh: cube.clone(),
            material: blue.clone(),
            transform: Transform {
                scale: DVec3::splat(1.0),
                translation: DVec3::ZERO,
                ..default()
            },
            ..default()
        })
        .insert(RigidBodyBundle::new_kinematic().with_pos(DVec3::new(0.0, 0.0, 0.0)))
        .insert(Player)
        .insert(MoveSpeed(0.3))
        .id();

    let object = commands
        .spawn(PbrBundle {
            mesh: cube,
            material: blue,
            transform: Transform {
                scale: DVec3::splat(1.0),
                translation: DVec3::ZERO,
                ..default()
            },
            ..default()
        })
        .insert(
            RigidBodyBundle::new_dynamic()
                .with_pos(DVec3::X * 1.5)
                .with_mass_props_from_shape(&Shape::cuboid(0.5, 0.5, 0.5), 1.0),
        )
        .id();

    commands.spawn(
        FixedJoint::new_with_compliance(anchor, object, 0.0).with_local_anchor_1(DVec3::X * 1.5),
    );

    // Directional 'sun' light
    let sun_half_size = 50.0;
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            // Configure the projection to better fit the scene
            shadow_projection: OrthographicProjection {
                left: -sun_half_size,
                right: sun_half_size,
                bottom: -sun_half_size,
                top: sun_half_size,
                near: -10.0 * sun_half_size,
                far: 10.0 * sun_half_size,
                ..default()
            },
            illuminance: 20_000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: DVec3::new(0.0, 10.0, 0.0),
            rotation: DQuat::from_euler(
                EulerRot::XYZ,
                std::f64::consts::PI * 2.3,
                std::f64::consts::PI * 2.05,
                0.0,
            ),
            ..default()
        },
        ..default()
    });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_translation(DVec3::new(0.0, 0.0, 10.0))
            .looking_at(DVec3::Y * 0.0, DVec3::Y),
        ..default()
    });
}

fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut LinVel, &mut AngVel, &MoveSpeed), With<Player>>,
) {
    for (mut lin_vel, mut ang_vel, move_speed) in &mut query {
        lin_vel.0 *= 0.95;
        ang_vel.0 *= 0.95;
        if keyboard_input.pressed(KeyCode::Up) {
            lin_vel.z -= move_speed.0;
        }
        if keyboard_input.pressed(KeyCode::Down) {
            lin_vel.z += move_speed.0;
        }
        if keyboard_input.pressed(KeyCode::Left) {
            lin_vel.x -= move_speed.0;
        }
        if keyboard_input.pressed(KeyCode::Right) {
            lin_vel.x += move_speed.0;
        }
        if keyboard_input.pressed(KeyCode::W) {
            lin_vel.y += move_speed.0 * 0.75;
        }
        if keyboard_input.pressed(KeyCode::S) {
            lin_vel.y -= move_speed.0 * 0.75;
        }
        if keyboard_input.pressed(KeyCode::Q) {
            ang_vel.0 += move_speed.0;
        }
        if keyboard_input.pressed(KeyCode::E) {
            ang_vel.0 -= move_speed.0;
        }
    }
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.1)))
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(Gravity(DVec3::Y * -9.81))
        .insert_resource(NumSubsteps(50))
        .add_plugins(DefaultPlugins)
        .add_plugin(XpbdExamplePlugin)
        .add_startup_system(setup)
        .add_system(player_movement)
        .run();
}
