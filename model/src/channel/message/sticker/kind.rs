use crate::visitor::NumericEnumVisitor;
use serde::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};

/// Format type of a [Sticker][`super::Sticker`].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StickerFormatType {
    /// Sticker format is a PNG.
    Png,
    /// Sticker format is an APNG.
    Apng,
    /// Sticker format is a LOTTIE.
    Lottie,
    /// Type is unknown to Twilight.
    Unknown {
        /// Raw unknown variant number.
        value: u8,
    },
}

impl StickerFormatType {
    /// Retrieve the raw API variant number.
    ///
    /// # Examples
    ///
    /// ```
    /// use twilight_model::channel::message::sticker::StickerFormatType;
    ///
    /// assert_eq!(2, StickerFormatType::Apng.number());
    /// ```
    pub fn number(self) -> u8 {
        match self {
            Self::Png => 1,
            Self::Apng => 2,
            Self::Lottie => 3,
            Self::Unknown { value } => value,
        }
    }
}

impl From<u8> for StickerFormatType {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Png,
            2 => Self::Apng,
            3 => Self::Lottie,
            value => Self::Unknown { value },
        }
    }
}

impl<'de> Deserialize<'de> for StickerFormatType {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_u8(NumericEnumVisitor::new("sticker format type"))
    }
}

impl Serialize for StickerFormatType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(self.number())
    }
}

#[cfg(test)]
mod tests {
    use super::StickerFormatType;
    use serde_test::Token;

    const MAP: &[(StickerFormatType, u8)] = &[
        (StickerFormatType::Png, 1),
        (StickerFormatType::Apng, 2),
        (StickerFormatType::Lottie, 3),
    ];

    #[test]
    fn test_variants() {
        for (kind, num) in MAP {
            serde_test::assert_tokens(kind, &[Token::U8(*num)]);
            assert_eq!(*kind, StickerFormatType::from(*num));
            assert_eq!(*num, kind.number());
        }
    }

    #[test]
    fn test_unknown_conversion() {
        assert_eq!(
            StickerFormatType::Unknown { value: 250 },
            StickerFormatType::from(250)
        );
    }
}
