use bevy::ecs::system::RunSystemOnce;
use bevy::gltf::Gltf;
use bevy::render::mesh::shape::{Circle, Plane};
use mw_app::camera::{GridCursor, GridCursorChangedSet};
use mw_common::grid::*;
use mw_app::map::*;

use crate::assets::ass3d::Ass3dConfig;
use crate::gfx3d::map::*;
use crate::prelude::*;
use super::*;
use super::asset_resolver::Ass3dResolver;

pub struct Gfx3dSimple3dPlugin;

impl Plugin for Gfx3dSimple3dPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            (setup_tilemap, setup_water, setup_cursor)
                .in_set(MapTopologySet(Topology::Hex))
                .in_set(TilemapSetupSet)
                .in_set(Gfx3dSet::Simple3D)
                .run_if(not(resource_exists::<TilemapInitted>())),
            update_cursor
                .in_set(MapTopologySet(Topology::Hex))
                .in_set(GridCursorChangedSet),
        ));
    }
}

#[derive(Component)]
struct CursorMesh;

fn setup_tilemap(
    world: &mut World,
) {
    // let scene = world.resource::<Assets<Gltf>>()
    //     .get(&world.resource::<GameAssets>().fallback_3d_gltf)
    //     .unwrap()
    //     .named_scenes.get("LOD1_MTN_0_0").unwrap().clone();
    let index = world.remove_resource::<MapTileIndex<Hex>>().unwrap();
    for (c, &e) in index.0.iter() {
        let translation = c.translation();
        let transform = Transform::from_xyz(
            (translation.x as f64 * 3f64.sqrt() * TILE_SCALE as f64 / 2.0) as f32,
            0.0,
            -translation.y * TILE_SCALE,
        );
        world.entity_mut(e).insert((
            TileAss3d {
                kind: Ass3dTileKind::Water,
                variant: Ass3dTileVariant::V6,
                rotation: 0,
                neighmask: 0b111111,
                subvariant: [0, 0, 0],
            },
            SceneBundle {
                transform,
                ..Default::default()
            },
        ));
    }
    world.insert_resource(index);
    world.run_system_once(super::map::compute_tile_ass3d);
    world.run_system_once(update_tile_scene);
    world.insert_resource(TilemapInitted);
    debug!("Initialized map using Simple3D renderer.");
}

fn setup_water(
    world: &mut World,
) {
    // TODO: make fancier water material
    let material = StandardMaterial {
        base_color: Color::rgba(0.5, 0.75, 1.0, 0.75),
        alpha_mode: AlphaMode::Blend,
        ..Default::default()
    };
    let plane = Plane::from_size(TILE_SCALE * 256.0);
    let handle_mesh = world.resource_mut::<Assets<Mesh>>().add(plane.into());
    let handle_material = world.resource_mut::<Assets<StandardMaterial>>().add(material);
    world.spawn((
        PbrBundle {
            mesh: handle_mesh,
            material: handle_material,
            // TODO: hightide/lowtide
            transform: Transform::from_xyz(0.0, -2.5, 0.0),
            ..Default::default()
        },
    ));
}

fn setup_cursor(
    world: &mut World,
) {
    let material = StandardMaterial {
        base_color: Color::rgba(0.0, 0.0, 0.0, 0.5),
        alpha_mode: AlphaMode::Blend,
        ..Default::default()
    };
    let plane = Circle {
        radius: TILE_SCALE / 2.0,
        vertices: 6,
    };
    let handle_mesh = world.resource_mut::<Assets<Mesh>>().add(plane.into());
    let handle_material = world.resource_mut::<Assets<StandardMaterial>>().add(material);
    world.spawn((
        CursorMesh,
        PbrBundle {
            mesh: handle_mesh,
            material: handle_material,
            transform: Transform::from_xyz(0.0, 0.125, 0.0)
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            ..Default::default()
        },
    ));
}

fn update_cursor(
    mut q_cursor: Query<&mut Transform, With<CursorMesh>>,
    crs: Res<GridCursor>,
) {
    if !crs.is_changed() {
        return;
    }

    let translation = Hex::from(crs.0).translation();
    let mut transform = q_cursor.single_mut();
    transform.translation.x =
        (translation.x as f64 * 3f64.sqrt() * TILE_SCALE as f64 / 2.0) as f32;
    transform.translation.z =
        -translation.y * TILE_SCALE;
}

fn update_tile_scene(
    ass_ass3d: Res<Assets<Ass3dConfig>>,
    ass_gltf: Res<Assets<Gltf>>,
    resolver: Res<Ass3dResolver>,
    mut q_tile: Query<
        (&mut Handle<Scene>, &mut Transform, &TileAss3d),
        Changed<TileAss3d>
    >,
) {
    for (mut scene_handle, mut xf, ass3d) in &mut q_tile {
        if let Some(resolved) = resolver.get_tile_asset(
            &ass_ass3d, &ass_gltf, crate::assets::ass3d::Ass3dLod::Lod1, ass3d
        ) {
            xf.scale = Vec3::splat(resolved.scale);
            xf.rotation = ass3d.rotation_quat();
            *scene_handle = resolved.scene;
        } else {
            *scene_handle = Default::default();
        }
    }
}
