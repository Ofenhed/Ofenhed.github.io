use strum::{AsRefStr, EnumString, VariantArray};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, AsRefStr, VariantArray, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum Tag {
    Tech,
    Review,
    Keyboards,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, AsRefStr, VariantArray, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum Locale {
    #[strum(serialize = "sv_SE")]
    Swedish,
    #[strum(serialize = "en_UK")]
    English,
    #[strum(serialize = "en_US")]
    EnglishSimplified,
}
