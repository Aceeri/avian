use bevy::prelude::*;
use bevy_xpbd_2d::prelude::*;
use examples_common_2d::XpbdExamplePlugin;

#[derive(Resource, Default)]
struct MouseWorldPos(DVec2);

#[derive(Component)]
struct FollowMouse;

#[derive(Component)]
struct GameCamera;

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let sphere = meshes.add(Mesh::from(shape::Icosphere {
        radius: 0.5,
        subdivisions: 4,
    }));

    let blue = materials.add(StandardMaterial {
        base_color: Color::rgb(0.2, 0.6, 0.8),
        unlit: true,
        ..default()
    });

    // Rope
    create_chain(&mut commands, sphere, blue, 160, 0.0, 0.06, 0.0);

    // Pendulum
    //create_chain(&mut commands, sphere, blue, 4, 1.0, 1.0, 0.0);

    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_translation(DVec3::new(0.0, 0.0, 100.0)),
            projection: OrthographicProjection {
                scale: 0.025,
                ..default()
            }
            .into(),
            ..default()
        })
        .insert(GameCamera);
}

fn create_chain(
    commands: &mut Commands,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    node_count: usize,
    node_dist: f64,
    node_size: f64,
    compliance: f64,
) {
    let mut prev = commands
        .spawn(PbrBundle {
            mesh: mesh.clone(),
            material: material.clone(),
            transform: Transform {
                scale: DVec3::splat(node_size),
                translation: DVec3::ZERO,
                ..default()
            },
            ..default()
        })
        .insert(RigidBodyBundle::new_kinematic())
        .insert(FollowMouse)
        .id();

    for i in 1..node_count {
        let curr = commands
            .spawn(PbrBundle {
                mesh: mesh.clone(),
                material: material.clone(),
                transform: Transform {
                    scale: DVec3::splat(node_size),
                    translation: DVec3::ZERO,
                    ..default()
                },
                ..default()
            })
            .insert(
                RigidBodyBundle::new_dynamic()
                    .with_pos(DVec2::Y * -(node_size + node_dist) * i as f64)
                    .with_mass_props_from_shape(&Shape::ball(node_size * 0.5), 1.0),
            )
            .id();

        commands.spawn(
            SphericalJoint::new_with_compliance(prev, curr, compliance)
                .with_local_anchor_2(DVec2::Y * (node_size + node_dist)),
        );

        prev = curr;
    }
}

fn mouse_position(
    windows: Res<Windows>,
    mut mouse_world_pos: ResMut<MouseWorldPos>,
    q_camera: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
) {
    let window = windows.get_primary().unwrap();
    let (camera, camera_transform) = q_camera.single();
    if let Some(pos) = window.cursor_position() {
        let window_size = DVec2::new(window.width() as f64, window.height() as f64);

        // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
        let ndc = (pos / window_size) * 2.0 - DVec2::ONE;

        // matrix for undoing the projection and camera transform
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();

        // use it to convert ndc to world-space coordinates
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

        mouse_world_pos.0 = world_pos.truncate();
    }
}

fn follow_mouse(
    mouse_pos: Res<MouseWorldPos>,
    buttons: Res<Input<MouseButton>>,
    mut query: Query<&mut Pos, With<FollowMouse>>,
) {
    for mut pos in &mut query {
        if buttons.pressed(MouseButton::Left) {
            pos.0 = mouse_pos.0;
        }
    }
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(Gravity(DVec2::new(0.0, -9.81)))
        .insert_resource(NumSubsteps(15))
        .insert_resource(NumPosIters(6))
        .init_resource::<MouseWorldPos>()
        .add_plugins(DefaultPlugins)
        .add_plugin(XpbdExamplePlugin)
        .add_startup_system(setup)
        .add_system(mouse_position)
        .add_system(follow_mouse)
        .run();
}
