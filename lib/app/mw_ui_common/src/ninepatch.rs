use crate::{assets::properties::NinePatchMargins, prelude::*};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (
        update_9p.run_if(any_with_component::<Handle<NinePatchMargins>>),
    ));
}

fn update_9p(
    mut commands: Commands,
    mut q_9p: Query<(&mut ImageScaleMode, &Handle<NinePatchMargins>)>,
    q_9p_new: Query<(Entity, &Handle<NinePatchMargins>), Without<ImageScaleMode>>,
    mut evr_asset: EventReader<AssetEvent<NinePatchMargins>>,
    assets: Res<Assets<NinePatchMargins>>,
    mut changeds: Local<HashSet<AssetId<NinePatchMargins>>>,
) {
    changeds.clear();
    changeds.extend(evr_asset.read().filter_map(|ev| match ev {
        AssetEvent::Modified { id } => Some(*id),
        _ => None,
    }));
    if !changeds.is_empty() {
        q_9p.iter_mut().for_each(|(mut mode, handle)| {
            if changeds.contains(&handle.id()) {
                if let Some(a) = assets.get(handle) {
                    *mode = a.into();
                }
            }
        });
    }
    q_9p_new.iter().for_each(|(e, handle)| {
        if let Some(a) = assets.get(handle) {
            commands.entity(e).insert(ImageScaleMode::from(a));
        }
    });
}
