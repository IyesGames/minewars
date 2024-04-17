use crate::{locale::{L10nApplySS, L10nKey}, prelude::*};

pub(super) struct TooltipPlugin;

impl Plugin for TooltipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            interact_infotext.before(SetStage::Prepare(L10nApplySS)),
        ));
    }
}

/// Marker for the text entity that dispays on-hover info / tooltips
#[derive(Component)]
pub(super) struct InfoAreaText;

/// This is the `L10nKey` that will be used for the tooltip
#[derive(Component)]
pub(super) struct InfoText(pub String);

fn interact_infotext(
    q_butt: Query<(&Interaction, &InfoText), Changed<Interaction>>,
    mut q_info: Query<&mut L10nKey, With<InfoAreaText>>,
) {
    let mut newtext = None;
    let mut clear = false;
    for (interaction, infotext) in &q_butt {
        match interaction {
            Interaction::None => {
                clear = true;
            }
            _ => {
                newtext = Some(&infotext.0);
            }
        }
    }
    if clear || newtext.is_some() {
        for mut infol10n in &mut q_info {
            if let Some(newtext) = newtext {
                infol10n.0 = String::from(newtext);
            } else {
                infol10n.0 = String::new();
            }
        }
    }
}
