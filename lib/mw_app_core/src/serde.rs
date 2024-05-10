use crate::prelude::*;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub struct Lcha(pub f32, pub f32, pub f32);

impl From<Color> for Lcha {
    fn from(value: Color) -> Self {
        let lcha = value.as_lcha_f32();
        Lcha(lcha[0], lcha[1], lcha[2])
    }
}

impl From<Lcha> for Color {
    fn from(value: Lcha) -> Self {
        Color::Lcha {
            lightness: value.0,
            chroma: value.1,
            hue: value.2,
            alpha: 1.0,
        }
    }
}
