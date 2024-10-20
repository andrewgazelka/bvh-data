#![allow(
    clippy::needless_pass_by_value,
    reason = "generally bevy wants to be able to pass resources by value"
)]
#![allow(clippy::cast_precision_loss, clippy::cast_lossless)]

use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bvh::{child_left, child_right, Aabb, Bvh, Point, ROOT_IDX};
use glam::I16Vec2;
use std::borrow::Cow;
use std::collections::VecDeque;

fn main() {
    println!("Starting application");

    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<BvhResource>()
        .add_systems(Startup, setup)
        .add_systems(Update, (pan_camera, zoom_camera))
        .run();
}

#[derive(Default, Resource)]
struct BvhResource {
    bvh: Option<Bvh<Vec<u8>>>,
}

#[derive(Component)]
struct BvhNode;

struct ChunkWithPackets {
    location: I16Vec2,
    packets_data: Vec<u8>,
}

impl Point for ChunkWithPackets {
    fn point(&self) -> I16Vec2 {
        self.location
    }
}

impl bvh::Data for ChunkWithPackets {
    type Unit = u8;

    fn data(&self) -> &[u8] {
        &self.packets_data
    }
}

fn setup(mut commands: Commands, mut bvh_resource: ResMut<BvhResource>) {
    println!("Setting up the scene");
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical(1000.0),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(150.0, 150.0, 999.9)), // Adjusted position
        ..default()
    });

    let bvh = generate_random_bvh(&mut fastrand::Rng::with_seed(3), 100.0, 1000, 4);
    println!("BVH {}", bvh.print());
    bvh_resource.bvh = Some(bvh);

    if let Some(bvh) = &bvh_resource.bvh {
        visualize_bvh(&mut commands, bvh);
    } else {
        println!("BVH is None, skipping visualization");
    }
}

fn visualize_bvh(commands: &mut Commands, bvh: &Bvh<Vec<u8>>) {
    println!("Starting BVH visualization");
    let mut queue = VecDeque::new();
    queue.push_back((ROOT_IDX, 0));

    while let Some((idx, depth)) = queue.pop_front() {
        let node = unsafe { bvh.get_node(idx) };
        if let Some(node) = node.into_expanded() {
            match node {
                bvh::node::Expanded::Aabb(aabb) => {
                    spawn_aabb(commands, aabb, depth, idx);
                    let left = child_left(idx);
                    let right = child_right(idx);
                    queue.push_back((left, depth + 1));
                    queue.push_back((right, depth + 1));
                }
                bvh::node::Expanded::Leaf(leaf) => {
                    spawn_leaf(commands, leaf.point, idx);
                }
            }
        }
    }
}

fn spawn_aabb(commands: &mut Commands, aabb: Aabb, depth: usize, idx: u32) {
    const MAX_DEPTH: usize = 20;
    let alpha = (depth) as f32 / MAX_DEPTH as f32;

    println!(
        "Spawning AABB node: idx={}, depth={}, min={:?}, max={:?}",
        idx, depth, aabb.min, aabb.max
    );

    let color = Color::rgba(1.0, 0.0, 0.0, alpha);

    let size = Vec2::new(
        (aabb.max.x - aabb.min.x) as f32,
        (aabb.max.y - aabb.min.y) as f32,
    );

    let position = Vec2::new(
        (aabb.min.x + aabb.max.x) as f32 / 2.0,
        (aabb.min.y + aabb.max.y) as f32 / 2.0,
    );

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_translation(position.extend(0.0)),
            ..default()
        },
        BvhNode,
    ));
    // .with_children(|parent| {
    //     parent.spawn(Text2dBundle {
    //         text: Text::from_section(
    //             format!("{idx:02}"),
    //             TextStyle {
    //                 font_size: 20.0,
    //                 color: Color::WHITE,
    //                 ..default()
    //             },
    //         ),
    //         transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
    //         ..default()
    //     });
    // });
}

fn spawn_leaf(commands: &mut Commands, point: I16Vec2, idx: u32) {
    println!("Spawning leaf node: idx={idx}, point={point:?}");
    // commands
    //     .spawn((
    //         SpriteBundle {
    //             sprite: Sprite {
    //                 color: RED,
    //                 custom_size: Some(Vec2::new(10.0, 10.0)), // Increased size for visibility
    //                 ..default()
    //             },
    //             transform: Transform::from_translation(Vec3::new(
    //                 point.x as f32,
    //                 point.y as f32,
    //                 0.0,
    //             )),
    //             ..default()
    //         },
    //         BvhNode,
    //     ))
    //     .with_children(|parent| {
    //         parent.spawn(Text2dBundle {
    //             text: Text::from_section(
    //                 format!("{idx:02}"),
    //                 TextStyle {
    //                     font_size: 5.0,
    //                     color: Color::WHITE,
    //                     ..default()
    //                 },
    //             ),
    //             transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
    //             ..default()
    //         });
    //     });

    commands.spawn(Text2dBundle {
        text: Text::from_section(
            format!("{idx:02}"),
            TextStyle {
                font_size: 50.0, // Large font size for clarity
                color: Color::WHITE,
                ..default()
            },
        ),
        transform: Transform {
            scale: Vec3::splat(0.01), // Scale it down to make the text smaller
            translation: Vec3::new(point.x as f32, point.y as f32, 0.0),
            ..default()
        },
        ..default()
    });
}

fn pan_camera(
    mut camera_query: Query<&mut Transform, With<Camera>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mouse_button: Res<ButtonInput<MouseButton>>,
) {
    if mouse_button.pressed(MouseButton::Left) {
        let mut camera_transform = camera_query.single_mut();
        for event in mouse_motion.read() {
            camera_transform.translation.x -= event.delta.x;
            camera_transform.translation.y += event.delta.y;
        }
        println!("Camera panned to: {:?}", camera_transform.translation);
    }
}

fn zoom_camera(
    mut camera_query: Query<&mut OrthographicProjection, With<Camera>>,
    mut scroll_evr: EventReader<MouseWheel>,
) {
    let mut projection = camera_query.single_mut();
    for event in scroll_evr.read() {
        projection.scale *= event.y.mul_add(-0.001, 1.0);
    }
}

fn generate_random_bvh(
    rng: &mut fastrand::Rng,
    radius: f32,
    num_packets: usize,
    packet_size: usize,
) -> Bvh<Vec<u8>> {
    let mut input = Vec::with_capacity(num_packets);

    for _ in 0..num_packets {
        let r = rng.f32() * radius;
        let theta = rng.f32() * 2.0 * std::f32::consts::PI;
        let x = (r * theta.cos()) as i16;
        let y = (r * theta.sin()) as i16;

        let packet_data: Vec<u8> = (0..packet_size).map(|_| rng.u8(..)).collect();

        input.push(ChunkWithPackets {
            location: I16Vec2::new(x, y),
            packets_data: packet_data,
        });
    }

    Bvh::build(input)
}
