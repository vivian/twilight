use crate::visitor::NumericEnumVisitor;
use serde::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MessageActivityType {
    Join,
    Spectate,
    Listen,
    JoinRequest,
    /// Type is unknown to Twilight.
    Unknown {
        /// Raw unknown variant number.
        value: u8,
    },
}

impl MessageActivityType {
    /// Retrieve the raw API variant number.
    ///
    /// # Examples
    ///
    /// ```
    /// use twilight_model::channel::message::MessageActivityType;
    ///
    /// assert_eq!(1, MessageActivityType::Join.number());
    /// ```
    pub fn number(self) -> u8 {
        match self {
            Self::Join => 1,
            Self::Spectate => 2,
            Self::Listen => 3,
            Self::JoinRequest => 5,
            Self::Unknown { value } => value,
        }
    }
}

impl From<u8> for MessageActivityType {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Join,
            2 => Self::Spectate,
            3 => Self::Listen,
            5 => Self::JoinRequest,
            value => Self::Unknown { value },
        }
    }
}

impl<'de> Deserialize<'de> for MessageActivityType {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_u8(NumericEnumVisitor::new("activity type"))
    }
}

impl Serialize for MessageActivityType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(self.number())
    }
}

#[cfg(test)]
mod tests {
    use super::MessageActivityType;
    use serde_test::Token;

    const MAP: &[(MessageActivityType, u8)] = &[
        (MessageActivityType::Join, 1),
        (MessageActivityType::Spectate, 2),
        (MessageActivityType::Listen, 3),
        (MessageActivityType::JoinRequest, 5),
    ];

    #[test]
    fn test_variants() {
        for (kind, num) in MAP {
            serde_test::assert_tokens(kind, &[Token::U8(*num)]);
            assert_eq!(*kind, MessageActivityType::from(*num));
            assert_eq!(*num, kind.number());
        }
    }

    #[test]
    fn test_unknown_conversion() {
        assert_eq!(
            MessageActivityType::Unknown { value: 250 },
            MessageActivityType::from(250)
        );
    }
}
