use std::f32::consts::PI;

use bevy::{
    camera::Exposure,
    core_pipeline::tonemapping::Tonemapping,
    gltf::GltfAssetLabel,
    light::{CascadeShadowConfigBuilder, light_consts::lux},
    pbr::{Atmosphere, AtmosphereSettings, ScatteringMedium},
    post_process::bloom::Bloom,
    prelude::*,
    render::view::Hdr,
    scene::SceneRoot, // Added missing imports
};
use bevy_ingame_clock::InGameClock;
use bevy_sun_move::{random_stars::*, *}; // Your library

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SunMovePlugin) // Your plugin
        .add_plugins(RandomStarsPlugin)
        .add_systems(Startup, (setup_camera_fog, setup_terrain_scene))
        .run();
}

fn setup_camera_fog(
    mut commands: Commands,
    mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
) {
    let bundle = (
        Camera3d::default(),
        Transform::from_xyz(-1.2, 0.15, 0.0).looking_at(Vec3::Y * 0.1, Vec3::Y),
        Camera { ..default() },
        // HDR is required for atmospheric scattering to be properly applied to the scene
        Hdr,
        Atmosphere::earthlike(scattering_mediums.add(ScatteringMedium::default())),
        AtmosphereSettings {
            aerial_view_lut_max_distance: 3.2e5,
            scene_units_to_m: 1e+4,
            ..Default::default()
        },
        Exposure::SUNLIGHT,
        Tonemapping::AcesFitted,
        Bloom::NATURAL,
    );
    commands.spawn(bundle);
}

#[derive(Component)]
struct Terrain;

// Spawn scene similar to the bevy github example
fn setup_terrain_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(InGameClock::new().with_day_duration(4320.0));
    // Configure a properly scaled cascade shadow map for this scene (defaults are too large, mesh units are in km)
    let cascade_shadow_config = CascadeShadowConfigBuilder {
        first_cascade_far_bound: 0.3,
        maximum_distance: 3.0,
        ..default()
    }
    .build();

    // Sun
    let sun_id = commands
        .spawn((
            DirectionalLight {
                shadows_enabled: true,
                illuminance: lux::RAW_SUNLIGHT, // Full sunlight illuminance
                ..default()
            },
            // Start position doesn't matter as update_sky_center will set it
            Transform::default(),
            cascade_shadow_config,
        ))
        .id();

    // -- Create the SkyCenter entity
    commands.spawn((
        SkyCenter {
            sun: sun_id,
            latitude_degrees: 51.5,    // Approximate latitude of London
            planet_tilt_degrees: 23.5, // Earth's axial tilt
            year_fraction: 0.0,
            cycle_duration_secs: 30.0, // A 30-second day
            current_cycle_time: 0,     // Start at midnight
            ..default()
        },
        Visibility::Visible,
        StarSpawner {
            star_count: 1000,
            spawn_radius: 5000.0,
        },
    ));

    let sphere_mesh = meshes.add(Mesh::from(Sphere { radius: 1.0 }));

    // light probe spheres (using Mesh3dBundle for convenience)
    commands.spawn((
        Mesh3d(sphere_mesh.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            metallic: 1.0,
            perceptual_roughness: 0.0,
            ..default()
        })),
        Transform::from_xyz(-0.3, 0.1, -0.1).with_scale(Vec3::splat(0.05)),
    ));

    commands.spawn((
        Mesh3d(sphere_mesh.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            metallic: 0.0,
            perceptual_roughness: 1.0,
            ..default()
        })),
        Transform::from_xyz(-0.3, 0.1, 0.1).with_scale(Vec3::splat(0.05)),
    ));

    // Terrain (using SceneBundle for convenience)
    commands.spawn((
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("terrain.glb"))),
        Transform::from_xyz(-1.0, 0.0, -0.5)
            .with_scale(Vec3::splat(0.5))
            .with_rotation(Quat::from_rotation_y(PI / 2.0)),
    ));

    // Add an origin marker sphere
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.02))),
        MeshMaterial3d(materials.add(Color::srgb(1.0, 0.0, 0.0))),
    ));
}
