//! Phoneme Encoding for Place Names

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Ph {
    Space,
    A,
    E,
    I,
    O,
    U,
    Ya,
    Ye,
    Yi,
    Yo,
    Yu,
    B,
    Ch,
    D,
    F,
    G,
    H,
    K,
    Kh,
    L,
    M,
    N,
    P,
    R,
    S,
    Sh,
    T,
    V,
    Z,
    Ts,
    Zh,
    Dj,
    Bb,
    Cch,
    Dd,
    Ff,
    Gg,
    Kk,
    Ll,
    Mm,
    Nn,
    Pp,
    Rr,
    Ss,
    Ssh,
    Tt,
    Vv,
    Zz,
}

pub trait RenderPhoneme<Lang> {
    fn render(self, prev: Self, next: Self, out: &mut String);
}

pub fn render_str<Lang>(phs: &[Ph]) -> String
where Ph: RenderPhoneme<Lang>,
{
    let mut s = String::new();
    for i in 0..phs.len() {
        let cur = phs[i];
        let prev = if i == 0 {
            Ph::Space
        } else {
            phs[i - 1]
        };
        let next = if i == phs.len() - 1 {
            Ph::Space
        } else {
            phs[i + 1]
        };
        cur.render(prev, next, &mut s);
    }
    s
}

impl From<Ph> for u8 {
    fn from(value: Ph) -> Self {
        value as u8
    }
}

impl Ph {
    pub fn is_vowel(self) -> bool {
        self.is_vowel_simple() || self.is_vowel_iotated()
    }
    pub fn is_vowel_simple(self) -> bool {
        match self {
            Ph::A | Ph::E | Ph::I | Ph::O | Ph::U => true,
            _ => false,
        }
    }
    pub fn is_vowel_iotated(self) -> bool {
        match self {
            Ph::Ya | Ph::Ye | Ph::Yi | Ph::Yo | Ph::Yu => true,
            _ => false,
        }
    }
    pub fn is_consonant_single(self) -> bool {
        match self {
            Ph::B | Ph::Ch | Ph::D | Ph::F | Ph::G | Ph::H | Ph::K | Ph::Kh | Ph::L | Ph::M | Ph::N | Ph::P | Ph::R | Ph::S | Ph::Sh | Ph::T | Ph::V | Ph::Z | Ph::Ts | Ph::Zh | Ph::Dj => true,
            _ => false,
        }
    }
    pub fn is_consonant_double(self) -> bool {
        match self {
            Ph::Bb | Ph::Cch | Ph::Dd | Ph::Ff | Ph::Gg | Ph::Kk | Ph::Ll | Ph::Mm | Ph::Nn | Ph::Pp | Ph::Rr | Ph::Ss | Ph::Ssh | Ph::Tt | Ph::Vv | Ph::Zz => true,
            _ => false,
        }
    }
}

pub mod lang {
    use super::*;

    /// English
    pub struct EN;
    /// Bulgarian
    pub struct BG;
    /// Russian
    pub struct RU;
    /// Ukrainian
    pub struct UK;
    /// Serbian
    pub struct SR;

    impl RenderPhoneme<lang::EN> for Ph {
        fn render(self, prev: Self, next: Self, out: &mut String) {
            out.push_str(match (prev, self, next) {
                (_, Ph::Space, _) => " ",
                (_, Ph::A, _) => "a",
                (_, Ph::E, _) => "e",
                (_, Ph::O, _) => "o",
                (_, Ph::U, _) => "u",
                (_, Ph::I, _) => "i",
                (_, Ph::Ya, _) => "ya",
                (_, Ph::Ye, _) => "ye",
                (_, Ph::Yo, _) => "yo",
                (_, Ph::Yu, _) => "yu",
                (_, Ph::Yi, _) => "yi",
                (_, Ph::B, _) => "b",
                (_, Ph::Bb, _) => "bb",
                (_, Ph::Ch, _) => "ch",
                (_, Ph::Cch, _) => "cch",
                (_, Ph::D, _) => "d",
                (_, Ph::Dd, _) => "dd",
                (_, Ph::F, _) => "f",
                (_, Ph::Ff, _) => "ff",
                (_, Ph::G, _) => "g",
                (_, Ph::Gg, _) => "gg",
                (_, Ph::H, _) => "h",
                (_, Ph::K, _) => "k",
                (_, Ph::Kk, _) => "kk",
                (_, Ph::Kh, _) => "kh",
                (_, Ph::L, _) => "l",
                (_, Ph::Ll, _) => "ll",
                (_, Ph::M, _) => "m",
                (_, Ph::Mm, _) => "mm",
                (_, Ph::N, _) => "n",
                (_, Ph::Nn, _) => "nn",
                (_, Ph::P, _) => "p",
                (_, Ph::Pp, _) => "pp",
                (_, Ph::R, _) => "r",
                (_, Ph::Rr, _) => "rr",
                (_, Ph::S, _) => "s",
                (_, Ph::Ss, _) => "ss",
                (_, Ph::Sh, _) => "sh",
                (_, Ph::Ssh, _) => "ssh",
                (_, Ph::T, _) => "t",
                (_, Ph::Tt, _) => "tt",
                (_, Ph::V, _) => "v",
                (_, Ph::Vv, _) => "vv",
                (_, Ph::Z, _) => "z",
                (_, Ph::Zz, _) => "zz",
                (_, Ph::Ts, _) => "ts",
                (_, Ph::Zh, _) => "zh",
                (_, Ph::Dj, Ph::Space) => "dge",
                (_, Ph::Dj, _) => "j",
            });
        }
    }

    impl RenderPhoneme<lang::BG> for Ph {
        fn render(self, prev: Self, next: Self, out: &mut String) {
            out.push_str(match (prev, self, next) {
                (_, Ph::Space, _) => " ",
                (_, Ph::A, _) => "а",
                (_, Ph::E, _) => "е",
                (_, Ph::O, _) => "о",
                (_, Ph::U, _) => "у",
                (_, Ph::I, _) => "и",
                (_, Ph::Ya, _) => "я",
                (Ph::Space, Ph::Ye, _) => "йе",
                (p, Ph::Ye, _) if p.is_vowel() => "йе",
                (_, Ph::Ye, _) => "ье",
                (Ph::Space, Ph::Yo, _) => "йо",
                (p, Ph::Yo, _) if p.is_vowel() => "йо",
                (_, Ph::Yo, _) => "ьо",
                (_, Ph::Yu, _) => "ю",
                (Ph::Space, Ph::Yi, _) => "йи",
                (p, Ph::Yi, _) if p.is_vowel() => "йи",
                (_, Ph::Yi, _) => "ьи",
                (_, Ph::B, _) => "б",
                (_, Ph::Bb, _) => "бб",
                (_, Ph::Ch, _) => "ч",
                (_, Ph::Cch, _) => "тч",
                (_, Ph::D, _) => "д",
                (_, Ph::Dd, _) => "дд",
                (_, Ph::F, _) => "ф",
                (_, Ph::Ff, _) => "фф",
                (_, Ph::G, _) => "г",
                (_, Ph::Gg, _) => "гг",
                (_, Ph::H, _) => "х",
                (_, Ph::K, _) => "к",
                (_, Ph::Kk, _) => "кк",
                (_, Ph::Kh, _) => "кх",
                (_, Ph::L, _) => "л",
                (_, Ph::Ll, _) => "лл",
                (_, Ph::M, _) => "м",
                (_, Ph::Mm, _) => "мм",
                (_, Ph::N, _) => "н",
                (_, Ph::Nn, _) => "нн",
                (_, Ph::P, _) => "п",
                (_, Ph::Pp, _) => "пп",
                (_, Ph::R, _) => "р",
                (_, Ph::Rr, _) => "рр",
                (_, Ph::S, _) => "с",
                (_, Ph::Ss, _) => "сс",
                (_, Ph::Sh, Ph::T) => "щ",
                (_, Ph::Sh, _) => "ш",
                (_, Ph::Ssh, _) => "шш",
                (Ph::Sh, Ph::T, _) => "",
                (_, Ph::T, _) => "т",
                (_, Ph::Tt, _) => "тт",
                (_, Ph::V, _) => "в",
                (_, Ph::Vv, _) => "вв",
                (_, Ph::Z, _) => "з",
                (_, Ph::Zz, _) => "зз",
                (_, Ph::Ts, _) => "ц",
                (_, Ph::Zh, _) => "ж",
                (_, Ph::Dj, _) => "дж",
            });
        }
    }

    impl RenderPhoneme<lang::RU> for Ph {
        fn render(self, prev: Self, next: Self, out: &mut String) {
            out.push_str(match (prev, self, next) {
                (_, Ph::Space, _) => " ",
                (_, Ph::A, _) => "а",
                (Ph::Zh, Ph::E, _) => "е",
                (Ph::Ts, Ph::E, _) => "е",
                (Ph::Sh, Ph::E, _) => "е",
                (Ph::Ssh, Ph::E, _) => "е",
                (Ph::Ch, Ph::E, _) => "е",
                (Ph::Cch, Ph::E, _) => "е",
                (_, Ph::E, _) => "э",
                (_, Ph::O, _) => "о",
                (_, Ph::U, _) => "у",
                (_, Ph::I, _) => "и",
                (_, Ph::Ya, _) => "я",
                (_, Ph::Ye, _) => "е",
                (_, Ph::Yo, _) => "ё",
                (_, Ph::Yu, _) => "ю",
                (Ph::Space, Ph::Yi, _) => "йи",
                (p, Ph::Yi, _) if p.is_vowel() => "йи",
                (_, Ph::Yi, _) => "ьи",
                (_, Ph::B, _) => "б",
                (_, Ph::Bb, _) => "бб",
                (_, Ph::Ch, Ph::Space) => "чь",
                (Ph::Sh, Ph::Ch, _) => "щ",
                (_, Ph::Ch, _) => "ч",
                (_, Ph::Cch, Ph::Space) => "тчь",
                (_, Ph::Cch, _) => "тч",
                (_, Ph::D, _) => "д",
                (_, Ph::Dd, _) => "дд",
                (_, Ph::F, _) => "ф",
                (_, Ph::Ff, _) => "фф",
                (_, Ph::G, _) => "г",
                (_, Ph::Gg, _) => "гг",
                (_, Ph::H, _) => "г",
                (_, Ph::K, _) => "к",
                (_, Ph::Kk, _) => "кк",
                (_, Ph::Kh, _) => "х",
                (_, Ph::L, Ph::Space) => "лл",
                (_, Ph::L, _) => "л",
                (_, Ph::Ll, _) => "лл",
                (_, Ph::M, _) => "м",
                (_, Ph::Mm, _) => "мм",
                (_, Ph::N, _) => "н",
                (_, Ph::Nn, _) => "нн",
                (_, Ph::P, _) => "п",
                (_, Ph::Pp, _) => "пп",
                (_, Ph::R, _) => "р",
                (_, Ph::Rr, _) => "рр",
                (_, Ph::S, _) => "с",
                (_, Ph::Ss, _) => "сс",
                (_, Ph::Sh, Ph::Ch) => "щ",
                (_, Ph::Sh, _) => "ш",
                (_, Ph::Ssh, _) => "шш",
                (_, Ph::T, _) => "т",
                (_, Ph::Tt, _) => "тт",
                (_, Ph::V, _) => "в",
                (_, Ph::Vv, _) => "вв",
                (_, Ph::Z, _) => "з",
                (_, Ph::Zz, _) => "зз",
                (_, Ph::Ts, _) => "ц",
                (_, Ph::Zh, _) => "ж",
                (_, Ph::Dj, _) => "дж",
            });
        }
    }

    impl RenderPhoneme<lang::UK> for Ph {
        fn render(self, prev: Self, next: Self, out: &mut String) {
            out.push_str(match (prev, self, next) {
                (_, Ph::Space, _) => " ",
                (_, Ph::A, _) => "а",
                (_, Ph::E, _) => "е",
                (_, Ph::O, _) => "о",
                (_, Ph::U, _) => "у",
                (_, Ph::I, _) => "і",
                (_, Ph::Ya, _) => "я",
                (_, Ph::Ye, _) => "є",
                (Ph::Space, Ph::Yo, _) => "йо",
                (p, Ph::Yo, _) if p.is_vowel() => "йо",
                (_, Ph::Yo, _) => "ьо",
                (_, Ph::Yu, _) => "ю",
                (_, Ph::Yi, _) => "ї",
                (_, Ph::B, _) => "б",
                (_, Ph::Bb, _) => "бб",
                (Ph::Sh, Ph::Ch, _) => "щ",
                (_, Ph::Ch, _) => "ч",
                (_, Ph::Cch, _) => "тч",
                (_, Ph::D, _) => "д",
                (_, Ph::Dd, _) => "дд",
                (_, Ph::F, _) => "ф",
                (_, Ph::Ff, _) => "фф",
                (_, Ph::G, _) => "ґ",
                (_, Ph::Gg, _) => "ґґ",
                (_, Ph::H, _) => "г",
                (_, Ph::K, _) => "к",
                (_, Ph::Kk, _) => "кк",
                (_, Ph::Kh, _) => "х",
                (_, Ph::L, _) => "л",
                (_, Ph::Ll, _) => "лл",
                (_, Ph::M, _) => "м",
                (_, Ph::Mm, _) => "мм",
                (_, Ph::N, _) => "н",
                (_, Ph::Nn, _) => "нн",
                (_, Ph::P, _) => "п",
                (_, Ph::Pp, _) => "пп",
                (_, Ph::R, _) => "р",
                (_, Ph::Rr, _) => "рр",
                (_, Ph::S, _) => "с",
                (_, Ph::Ss, _) => "сс",
                (_, Ph::Sh, Ph::Ch) => "щ",
                (_, Ph::Sh, _) => "ш",
                (_, Ph::Ssh, _) => "шш",
                (_, Ph::T, _) => "т",
                (_, Ph::Tt, _) => "тт",
                (_, Ph::V, _) => "в",
                (_, Ph::Vv, _) => "вв",
                (_, Ph::Z, _) => "з",
                (_, Ph::Zz, _) => "зз",
                (_, Ph::Ts, _) => "ц",
                (_, Ph::Zh, _) => "ж",
                (_, Ph::Dj, _) => "дж",
            });
        }
    }

    impl RenderPhoneme<lang::SR> for Ph {
        fn render(self, prev: Self, next: Self, out: &mut String) {
            out.push_str(match (prev, self, next) {
                (_, Ph::Space, _) => " ",
                (_, Ph::A, _) => "а",
                (_, Ph::E, _) => "е",
                (_, Ph::O, _) => "о",
                (_, Ph::U, _) => "у",
                (_, Ph::I, _) => "и",
                (Ph::N, v, _) if v.is_vowel_iotated() => "",
                (Ph::L, v, _) if v.is_vowel_iotated() => "",
                (Ph::Nn, v, _) if v.is_vowel_iotated() => "",
                (Ph::Ll, v, _) if v.is_vowel_iotated() => "",
                (_, Ph::Ya, _) => "ја",
                (_, Ph::Ye, _) => "је",
                (_, Ph::Yo, _) => "јо",
                (_, Ph::Yu, _) => "ју",
                (_, Ph::Yi, _) => "ји",
                (_, Ph::B, _) => "б",
                (_, Ph::Bb, _) => "бб",
                (_, Ph::Ch, _) => "ћ",
                (_, Ph::Cch, _) => "ч",
                (_, Ph::D, _) => "д",
                (_, Ph::Dd, _) => "дд",
                (_, Ph::F, _) => "ф",
                (_, Ph::Ff, _) => "фф",
                (_, Ph::G, _) => "г",
                (_, Ph::Gg, _) => "гг",
                (_, Ph::H, _) => "х",
                (_, Ph::K, _) => "к",
                (_, Ph::Kk, _) => "кк",
                (_, Ph::Kh, _) => "кх",
                (_, Ph::L, v) if v.is_vowel_iotated() => "љ",
                (_, Ph::L, _) => "л",
                (_, Ph::Ll, v) if v.is_vowel_iotated() => "лљ",
                (_, Ph::Ll, _) => "лл",
                (_, Ph::M, _) => "м",
                (_, Ph::Mm, _) => "мм",
                (_, Ph::N, v) if v.is_vowel_iotated() => "њ",
                (_, Ph::N, _) => "н",
                (_, Ph::Nn, v) if v.is_vowel_iotated() => "нњ",
                (_, Ph::Nn, _) => "нн",
                (_, Ph::P, _) => "п",
                (_, Ph::Pp, _) => "пп",
                (_, Ph::R, _) => "р",
                (_, Ph::Rr, _) => "рр",
                (_, Ph::S, _) => "с",
                (_, Ph::Ss, _) => "сс",
                (_, Ph::Sh, _) => "ш",
                (_, Ph::Ssh, _) => "шш",
                (_, Ph::T, _) => "т",
                (_, Ph::Tt, _) => "тт",
                (_, Ph::V, _) => "в",
                (_, Ph::Vv, _) => "вв",
                (_, Ph::Z, _) => "з",
                (_, Ph::Zz, _) => "зз",
                (_, Ph::Ts, _) => "ц",
                (_, Ph::Zh, _) => "ж",
                (_, Ph::Dj, _) => "ђ",
            });
        }
    }
}
