use crate::visitor::NumericEnumVisitor;
use serde::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MessageType {
    Regular,
    RecipientAdd,
    RecipientRemove,
    Call,
    ChannelNameChange,
    ChannelIconChange,
    ChannelMessagePinned,
    GuildMemberJoin,
    UserPremiumSub,
    UserPremiumSubTier1,
    UserPremiumSubTier2,
    UserPremiumSubTier3,
    ChannelFollowAdd,
    GuildDiscoveryDisqualified,
    GuildDiscoveryRequalified,
    GuildDiscoveryGracePeriodInitialWarning,
    GuildDiscoveryGracePeriodFinalWarning,
    /// Message is an inline reply.
    Reply,
    GuildInviteReminder,
    /// Type is unknown to Twilight.
    Unknown {
        /// Raw unknown variant number.
        value: u8,
    },
}

impl MessageType {
    /// Retrieve the raw API variant number.
    ///
    /// # Examples
    ///
    /// ```
    /// use twilight_model::channel::message::MessageType;
    ///
    /// assert_eq!(7, MessageType::GuildMemberJoin.number());
    /// ```
    pub fn number(self) -> u8 {
        match self {
            Self::Regular => 0,
            Self::RecipientAdd => 1,
            Self::RecipientRemove => 2,
            Self::Call => 3,
            Self::ChannelNameChange => 4,
            Self::ChannelIconChange => 5,
            Self::ChannelMessagePinned => 6,
            Self::GuildMemberJoin => 7,
            Self::UserPremiumSub => 8,
            Self::UserPremiumSubTier1 => 9,
            Self::UserPremiumSubTier2 => 10,
            Self::UserPremiumSubTier3 => 11,
            Self::ChannelFollowAdd => 12,
            Self::GuildDiscoveryDisqualified => 14,
            Self::GuildDiscoveryRequalified => 15,
            Self::GuildDiscoveryGracePeriodInitialWarning => 16,
            Self::GuildDiscoveryGracePeriodFinalWarning => 17,
            Self::Reply => 19,
            Self::GuildInviteReminder => 22,
            Self::Unknown { value } => value,
        }
    }
}

impl From<u8> for MessageType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Regular,
            1 => Self::RecipientAdd,
            2 => Self::RecipientRemove,
            3 => Self::Call,
            4 => Self::ChannelNameChange,
            5 => Self::ChannelIconChange,
            6 => Self::ChannelMessagePinned,
            7 => Self::GuildMemberJoin,
            8 => Self::UserPremiumSub,
            9 => Self::UserPremiumSubTier1,
            10 => Self::UserPremiumSubTier2,
            11 => Self::UserPremiumSubTier3,
            12 => Self::ChannelFollowAdd,
            14 => Self::GuildDiscoveryDisqualified,
            15 => Self::GuildDiscoveryRequalified,
            16 => Self::GuildDiscoveryGracePeriodInitialWarning,
            17 => Self::GuildDiscoveryGracePeriodFinalWarning,
            19 => Self::Reply,
            22 => Self::GuildInviteReminder,
            value => Self::Unknown { value },
        }
    }
}

impl<'de> Deserialize<'de> for MessageType {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_u8(NumericEnumVisitor::new("message type"))
    }
}

impl Serialize for MessageType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(self.number())
    }
}

#[cfg(test)]
mod tests {
    use super::MessageType;
    use serde_test::Token;

    const MAP: &[(MessageType, u8)] = &[
        (MessageType::Regular, 0),
        (MessageType::RecipientAdd, 1),
        (MessageType::RecipientRemove, 2),
        (MessageType::Call, 3),
        (MessageType::ChannelNameChange, 4),
        (MessageType::ChannelIconChange, 5),
        (MessageType::ChannelMessagePinned, 6),
        (MessageType::GuildMemberJoin, 7),
        (MessageType::UserPremiumSub, 8),
        (MessageType::UserPremiumSubTier1, 9),
        (MessageType::UserPremiumSubTier2, 10),
        (MessageType::UserPremiumSubTier3, 11),
        (MessageType::ChannelFollowAdd, 12),
        (MessageType::GuildDiscoveryDisqualified, 14),
        (MessageType::GuildDiscoveryRequalified, 15),
        (MessageType::GuildDiscoveryGracePeriodInitialWarning, 16),
        (MessageType::GuildDiscoveryGracePeriodFinalWarning, 17),
        (MessageType::Reply, 19),
        (MessageType::GuildInviteReminder, 22),
    ];

    #[test]
    fn test_variants() {
        for (kind, num) in MAP {
            serde_test::assert_tokens(kind, &[Token::U8(*num)]);
            assert_eq!(*kind, MessageType::from(*num));
            assert_eq!(*num, kind.number());
        }
    }

    #[test]
    fn test_unknown_conversion() {
        assert_eq!(MessageType::Unknown { value: 250 }, MessageType::from(250));
    }
}
