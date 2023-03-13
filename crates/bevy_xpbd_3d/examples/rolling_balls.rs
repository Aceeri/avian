use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;
use examples_common_3d::XpbdExamplePlugin;

#[derive(Component)]
struct Player;

#[derive(Component, Deref, DerefMut)]
pub struct RollAcceleration(pub DVec3);

#[derive(Component, Deref, DerefMut)]
pub struct MaxAngularVelocity(pub DVec3);

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let ball = meshes.add(Mesh::from(shape::Icosphere {
        radius: 0.5,
        subdivisions: 4,
    }));
    let cube = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));

    let white = materials.add(StandardMaterial {
        base_color: Color::rgb(0.8, 0.8, 1.0),
        ..default()
    });

    let blue = materials.add(StandardMaterial {
        base_color: Color::rgb(0.2, 0.6, 0.8),
        ..default()
    });

    let floor_size = DVec3::new(80.0, 1.0, 80.0);
    let _floor = commands
        .spawn(PbrBundle {
            mesh: cube,
            material: white,
            transform: Transform::from_scale(floor_size),
            ..default()
        })
        .insert(RigidBodyBundle::new_static().with_pos(DVec3::new(0.0, -5.0, 0.0)))
        .insert(ColliderBundle::new(
            &Shape::cuboid(floor_size.x * 0.5, floor_size.y * 0.5, floor_size.z * 0.5),
            1.0,
        ));

    let radius = 0.5;
    let count_x = 1;
    let count_y = 500;
    let count_z = 1;
    for y in 0..count_y {
        for x in 0..count_x {
            for z in 0..count_z {
                let pos = DVec3::new(
                    (x as f64 - count_x as f64 * 0.5) * 2.5 * radius,
                    2.5 * radius * y as f64,
                    (z as f64 - count_z as f64 * 0.5) * 2.5 * radius,
                );
                commands
                    .spawn(PbrBundle {
                        mesh: ball.clone(),
                        material: blue.clone(),
                        transform: Transform {
                            scale: DVec3::splat(radius * 2.0),
                            translation: pos,
                            ..default()
                        },
                        ..default()
                    })
                    .insert(RigidBodyBundle::new_dynamic().with_pos(pos))
                    .insert(ColliderBundle::new(&Shape::ball(radius), 1.0))
                    .insert(Player)
                    .insert(RollAcceleration(DVec3::splat(0.5)))
                    .insert(MaxAngularVelocity(DVec3::new(30.0, 30.0, 30.0)));
            }
        }
    }

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
                std::f64::consts::PI * 1.3,
                std::f64::consts::PI * 1.85,
                0.0,
            ),
            ..default()
        },
        ..default()
    });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_translation(DVec3::new(0.0, 15.0, -50.0))
            .looking_at(DVec3::Y * 10.0, DVec3::Y),
        ..default()
    });
}

fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut AngVel, &MaxAngularVelocity, &RollAcceleration), With<Player>>,
) {
    for (mut ang_vel, max_ang_vel, move_acceleration) in &mut query {
        if keyboard_input.pressed(KeyCode::Up) {
            ang_vel.x += move_acceleration.x;
        }
        if keyboard_input.pressed(KeyCode::Down) {
            ang_vel.x -= move_acceleration.x;
        }
        if keyboard_input.pressed(KeyCode::Left) {
            ang_vel.z -= move_acceleration.z;
        }
        if keyboard_input.pressed(KeyCode::Right) {
            ang_vel.z += move_acceleration.z;
        }
        ang_vel.0 = ang_vel.0.clamp(-max_ang_vel.0, max_ang_vel.0);
    }
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(NumSubsteps(6))
        .insert_resource(Gravity(DVec3::Y * -9.81))
        .add_plugins(DefaultPlugins)
        .add_plugin(XpbdExamplePlugin)
        .add_startup_system(setup)
        .add_system(player_movement)
        .run();
}
