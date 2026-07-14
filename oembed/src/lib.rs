#[cfg(feature = "macro")]
pub use oembed_macro::*;
pub use oembed_type::*;

#[cfg(all(test, feature = "macro"))]
mod test {
    use super::*;
    use std::borrow::Cow;
    #[test]
    fn never_gonna_give_you_up() {
        let data = oembed!(
            "https://www.youtube.com/oembed",
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ"
        );
        assert_eq!(
            data.title,
            Some(Cow::Borrowed(
                "Rick Astley - Never Gonna Give You Up (Official Video) (4K Remaster)"
            ))
        );
    }
    #[test]
    fn concat() {
        macro_rules! youtube {
            ($id:literal) => {
                oembed! {
                    "https://www.youtube.com/oembed",
                    "https://www.youtube.com/watch?v=" + $id
                }
            };
        }
        let data = youtube!("dQw4w9WgXcQ");
        assert_eq!(
            data.title,
            Some(Cow::Borrowed(
                "Rick Astley - Never Gonna Give You Up (Official Video) (4K Remaster)"
            ))
        );
    }
}
