use crate::visitor::NumericEnumVisitor;
use serde::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ChannelType {
    GuildText,
    Private,
    GuildVoice,
    Group,
    GuildCategory,
    GuildNews,
    GuildStore,
    GuildStageVoice,
    /// Type is unknown to Twilight.
    Unknown {
        /// Raw unknown variant number.
        value: u8,
    },
}

impl ChannelType {
    pub fn name(self) -> &'static str {
        match self {
            Self::Group => "Group",
            Self::GuildCategory => "GuildCategory",
            Self::GuildNews => "GuildNews",
            Self::GuildStageVoice => "GuildStageVoice",
            Self::GuildStore => "GuildStore",
            Self::GuildText => "GuildText",
            Self::GuildVoice => "GuildVoice",
            Self::Private => "Private",
            Self::Unknown { .. } => "Unknown",
        }
    }

    /// Retrieve the raw API variant number.
    ///
    /// # Examples
    ///
    /// ```
    /// use twilight_model::channel::ChannelType;
    ///
    /// assert_eq!(5, ChannelType::GuildNews.number());
    /// ```
    pub fn number(self) -> u8 {
        match self {
            Self::GuildText => 0,
            Self::Private => 1,
            Self::GuildVoice => 2,
            Self::Group => 3,
            Self::GuildCategory => 4,
            Self::GuildNews => 5,
            Self::GuildStore => 6,
            Self::GuildStageVoice => 13,
            Self::Unknown { value } => value,
        }
    }
}

impl From<u8> for ChannelType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::GuildText,
            1 => Self::Private,
            2 => Self::GuildVoice,
            3 => Self::Group,
            4 => Self::GuildCategory,
            5 => Self::GuildNews,
            6 => Self::GuildStore,
            13 => Self::GuildStageVoice,
            value => Self::Unknown { value },
        }
    }
}

impl<'de> Deserialize<'de> for ChannelType {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_u8(NumericEnumVisitor::new("channel type"))
    }
}

impl Serialize for ChannelType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(self.number())
    }
}

#[cfg(test)]
mod tests {
    use super::ChannelType;
    use serde_test::Token;

    const MAP: &[(ChannelType, u8)] = &[
        (ChannelType::GuildText, 0),
        (ChannelType::Private, 1),
        (ChannelType::GuildVoice, 2),
        (ChannelType::Group, 3),
        (ChannelType::GuildCategory, 4),
        (ChannelType::GuildNews, 5),
        (ChannelType::GuildStore, 6),
        (ChannelType::GuildStageVoice, 13),
    ];

    #[test]
    fn test_variants() {
        for (kind, num) in MAP {
            serde_test::assert_tokens(kind, &[Token::U8(*num)]);
            assert_eq!(*kind, ChannelType::from(*num));
            assert_eq!(*num, kind.number());
        }
    }

    #[test]
    fn test_unknown_conversion() {
        assert_eq!(ChannelType::Unknown { value: 250 }, ChannelType::from(250));
    }

    #[test]
    fn test_names() {
        assert_eq!("Group", ChannelType::Group.name());
        assert_eq!("GuildCategory", ChannelType::GuildCategory.name());
        assert_eq!("GuildNews", ChannelType::GuildNews.name());
        assert_eq!("GuildStageVoice", ChannelType::GuildStageVoice.name());
        assert_eq!("GuildStore", ChannelType::GuildStore.name());
        assert_eq!("GuildText", ChannelType::GuildText.name());
        assert_eq!("GuildVoice", ChannelType::GuildVoice.name());
        assert_eq!("Private", ChannelType::Private.name());
    }
}
