use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(Reflect)]
pub struct Lch(pub f32, pub f32, pub f32);

impl From<Color> for Lch {
    fn from(value: Color) -> Self {
        let lcha = value.as_lcha_f32();
        Lch(lcha[0], lcha[1], lcha[2])
    }
}

impl From<Lch> for Color {
    fn from(value: Lch) -> Self {
        Color::Lcha {
            lightness: value.0,
            chroma: value.1,
            hue: value.2,
            alpha: 1.0,
        }
    }
}
