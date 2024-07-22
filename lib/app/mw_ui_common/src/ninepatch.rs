use crate::{assets::properties::NinePatchMargins, prelude::*};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (
        update_9p.run_if(any_with_component::<Handle<NinePatchMargins>>),
    ));
}

fn update_9p(
    mut commands: Commands,
    mut q_9p: Query<(Entity, Option<&mut ImageScaleMode>, &Handle<NinePatchMargins>)>,
    mut evr_asset: EventReader<AssetEvent<NinePatchMargins>>,
    assets: Res<Assets<NinePatchMargins>>,
    mut changeds: Local<HashSet<AssetId<NinePatchMargins>>>,
) {
    changeds.clear();
    changeds.extend(evr_asset.read().filter_map(|ev| match ev {
        AssetEvent::Modified { id } => Some(*id),
        _ => None,
    }));
    for (e, mode, handle) in &mut q_9p {
        if let Some(mut mode) = mode {
            if changeds.contains(&handle.id()) {
                if let Some(a) = assets.get(handle) {
                    *mode = a.into();
                }
            }
        } else {
            if let Some(a) = assets.get(handle) {
                commands.entity(e).insert(ImageScaleMode::from(a));
            }
        }
    }
}
