use strum::{AsRefStr, EnumString, VariantArray};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, AsRefStr, VariantArray, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum Tag {
    Tech,
    Review,
    Keyboards,
}
