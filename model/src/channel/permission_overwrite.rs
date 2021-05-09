use crate::{
    guild::Permissions,
    id::{RoleId, UserId},
    visitor::NumericEnumVisitor,
};
use serde::{
    de::Deserializer,
    ser::SerializeStruct,
    Deserialize, Serialize, Serializer,
};

pub(crate) mod integer {
    use serde::de::{Deserializer, Error as DeError, Visitor};
    use std::{
        fmt::{Formatter, Result as FmtResult},
        marker::PhantomData,
    };

    struct IdVisitor(PhantomData<u64>);

    impl<'de> Visitor<'de> for IdVisitor {
        type Value = u64;

        fn expecting(&self, f: &mut Formatter<'_>) -> FmtResult {
            f.write_str("string or integer snowflake")
        }

        fn visit_u64<E: DeError>(self, value: u64) -> Result<Self::Value, E> {
            Ok(value)
        }

        fn visit_str<E: DeError>(self, value: &str) -> Result<Self::Value, E> {
            value.parse().map_err(DeError::custom)
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
        deserializer.deserialize_any(IdVisitor(PhantomData))
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PermissionOverwrite {
    pub allow: Permissions,
    pub deny: Permissions,
    pub kind: PermissionOverwriteType,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum PermissionOverwriteType {
    Member(UserId),
    Role(RoleId),
}

#[derive(Deserialize)]
struct PermissionOverwriteData {
    allow: Permissions,
    deny: Permissions,
    #[serde(deserialize_with = "integer::deserialize")]
    id: u64,
    #[serde(rename = "type")]
    kind: PermissionOverwriteTargetType,
}

/// Type of a permission overwrite target.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PermissionOverwriteTargetType {
    /// Permission overwrite targets an individual role.
    Role,
    /// Permission overwrite targets an individual member.
    Member,
}

impl PermissionOverwriteTargetType {
    /// Retrieve the raw API variant number.
    ///
    /// # Examples
    ///
    /// ```
    /// use twilight_model::channel::permission_overwrite::PermissionOverwriteTargetType;
    ///
    /// assert_eq!(1, PermissionOverwriteTargetType::Role.number());
    /// ```
    pub fn number(self) -> u8 {
        match self {
            Self::Role => 0,
            Self::Member => 1,
        }
    }
}

impl From<u8> for PermissionOverwriteTargetType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Role,
            1 => Self::Member,
            _ => todo!("needs an other variant"),
        }
    }
}

impl<'de> Deserialize<'de> for PermissionOverwriteTargetType {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_u8(NumericEnumVisitor::new("permission overwrite target type"))
    }
}

impl Serialize for PermissionOverwriteTargetType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(self.number())
    }
}

impl<'de> Deserialize<'de> for PermissionOverwrite {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let data = PermissionOverwriteData::deserialize(deserializer)?;

        let span = tracing::trace_span!("deserializing permission overwrite");
        let _span_enter = span.enter();

        let kind = match data.kind {
            PermissionOverwriteTargetType::Member => {
                let id = UserId(data.id);
                tracing::trace!(id = %id.0, kind = ?data.kind);

                PermissionOverwriteType::Member(id)
            }
            PermissionOverwriteTargetType::Role => {
                let id = RoleId(data.id);
                tracing::trace!(id = %id.0, kind = ?data.kind);

                PermissionOverwriteType::Role(id)
            }
        };

        Ok(Self {
            allow: data.allow,
            deny: data.deny,
            kind,
        })
    }
}

impl Serialize for PermissionOverwrite {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("PermissionOverwriteData", 4)?;

        state.serialize_field("allow", &self.allow)?;
        state.serialize_field("deny", &self.deny)?;

        match &self.kind {
            PermissionOverwriteType::Member(id) => {
                state.serialize_field("id", &id.0.to_string())?;
                state.serialize_field("type", &(PermissionOverwriteTargetType::Member.number()))?;
            }
            PermissionOverwriteType::Role(id) => {
                state.serialize_field("id", &id.0.to_string())?;
                state.serialize_field("type", &(PermissionOverwriteTargetType::Role.number()))?;
            }
        }

        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PermissionOverwrite, PermissionOverwriteTargetType, PermissionOverwriteType, Permissions,
    };
    use crate::id::UserId;
    use serde_test::Token;

    #[test]
    fn test_overwrite() {
        let overwrite = PermissionOverwrite {
            allow: Permissions::CREATE_INVITE,
            deny: Permissions::KICK_MEMBERS,
            kind: PermissionOverwriteType::Member(UserId(12_345_678)),
        };

        // We can't use serde_test because it doesn't support 128 bit integers.
        //
        // <https://github.com/serde-rs/serde/issues/1281>
        let input = r#"{
  "allow": "1",
  "deny": "2",
  "id": "12345678",
  "type": 1
}"#;

        assert_eq!(
            serde_json::from_str::<PermissionOverwrite>(input).unwrap(),
            overwrite
        );
        assert_eq!(serde_json::to_string_pretty(&overwrite).unwrap(), input);
    }

    #[test]
    fn test_blank_overwrite() {
        // Test integer deser used in guild templates.
        let raw = r#"{
  "allow": "1",
  "deny": "2",
  "id": 0,
  "type": 1
}"#;

        let value = PermissionOverwrite {
            allow: Permissions::CREATE_INVITE,
            deny: Permissions::KICK_MEMBERS,
            kind: PermissionOverwriteType::Member(UserId(0)),
        };

        let deserialized = serde_json::from_str::<PermissionOverwrite>(raw).unwrap();

        assert_eq!(deserialized, value);

        serde_test::assert_tokens(
            &value,
            &[
                Token::Struct {
                    name: "PermissionOverwriteData",
                    len: 4,
                },
                Token::Str("allow"),
                Token::Str("1"),
                Token::Str("deny"),
                Token::Str("2"),
                Token::Str("id"),
                Token::Str("0"),
                Token::Str("type"),
                Token::U8(1),
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn test_overwrite_type_name() {
        serde_test::assert_tokens(&PermissionOverwriteTargetType::Member, &[Token::U8(1)]);
        serde_test::assert_tokens(&PermissionOverwriteTargetType::Role, &[Token::U8(0)]);
    }
}
