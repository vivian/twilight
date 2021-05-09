use crate::visitor::NumericEnumVisitor;
use serde::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};

#[derive(
    Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord,
)]
pub enum WebhookType {
    Incoming,
    ChannelFollower,
    /// Type is unknown to Twilight.
    Unknown {
        /// Raw unknown variant number.
        value: u8,
    },
}

impl Default for WebhookType {
    fn default() -> Self {
        Self::Incoming
    }
}

impl WebhookType {
    /// Retrieve the raw API variant number.
    ///
    /// # Examples
    ///
    /// ```
    /// use twilight_model::channel::WebhookType;
    ///
    /// assert_eq!(1, WebhookType::Incoming.number());
    /// ```
    pub fn number(self) -> u8 {
        match self {
            Self::Incoming => 1,
            Self::ChannelFollower => 2,
            Self::Unknown { value } => value,
        }
    }
}

impl From<u8> for WebhookType {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Incoming,
            2 => Self::ChannelFollower,
            value => Self::Unknown { value },
        }
    }
}

impl<'de> Deserialize<'de> for WebhookType {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_u8(NumericEnumVisitor::new("webhook type"))
    }
}

impl Serialize for WebhookType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(self.number())
    }
}

#[cfg(test)]
mod tests {
    use super::WebhookType;
    use serde_test::Token;

    const MAP: &[(WebhookType, u8)] = &[
        (WebhookType::Incoming, 1),
        (WebhookType::ChannelFollower, 2),
    ];

    #[test]
    fn test_default() {
        assert_eq!(WebhookType::Incoming, WebhookType::default());
    }

    #[test]
    fn test_variants() {
        for (kind, num) in MAP {
            serde_test::assert_tokens(kind, &[Token::U8(*num)]);
            assert_eq!(*kind, WebhookType::from(*num));
            assert_eq!(*num, kind.number());
        }
    }

    #[test]
    fn test_unknown_conversion() {
        assert_eq!(
            WebhookType::Unknown { value: 250 },
            WebhookType::from(250)
        );
    }
}
