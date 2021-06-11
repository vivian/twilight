use crate::{
    client::Client,
    error::Error as HttpError,
    request::{self, validate, AuditLogReason, AuditLogReasonError, PendingResponse, Request},
    response::{marker::MemberBody, Response},
    routing::Route,
};
use serde::Serialize;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use twilight_model::id::{ChannelId, GuildId, RoleId, UserId};

/// The error created when the member can not be updated as configured.
#[derive(Debug)]
pub struct UpdateGuildMemberError {
    kind: UpdateGuildMemberErrorType,
}

impl UpdateGuildMemberError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &UpdateGuildMemberErrorType {
        &self.kind
    }

    /// Consume the error, returning the source error if there is any.
    #[allow(clippy::unused_self)]
    #[must_use = "consuming the error and retrieving the source has no effect if left unused"]
    pub fn into_source(self) -> Option<Box<dyn Error + Send + Sync>> {
        None
    }

    /// Consume the error, returning the owned error type and the source error.
    #[must_use = "consuming the error into its parts has no effect if left unused"]
    pub fn into_parts(
        self,
    ) -> (
        UpdateGuildMemberErrorType,
        Option<Box<dyn Error + Send + Sync>>,
    ) {
        (self.kind, None)
    }
}

impl Display for UpdateGuildMemberError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            UpdateGuildMemberErrorType::NicknameInvalid { .. } => {
                f.write_str("the nickname length is invalid")
            }
        }
    }
}

impl Error for UpdateGuildMemberError {}

/// Type of [`UpdateGuildMemberError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum UpdateGuildMemberErrorType {
    /// The nickname is either empty or the length is more than 32 UTF-16 characters.
    NicknameInvalid { nickname: String },
}

#[derive(Default, Serialize)]
struct UpdateGuildMemberFields {
    #[allow(clippy::option_option)]
    #[serde(skip_serializing_if = "Option::is_none")]
    channel_id: Option<Option<ChannelId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    deaf: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mute: Option<bool>,
    #[allow(clippy::option_option)]
    #[serde(skip_serializing_if = "Option::is_none")]
    nick: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    roles: Option<Vec<RoleId>>,
}

/// Update a guild member.
///
/// All fields are optional. Refer to [the discord docs] for more information.
///
/// [the discord docs]: https://discord.com/developers/docs/resources/guild#modify-guild-member
pub struct UpdateGuildMember<'a> {
    fields: UpdateGuildMemberFields,
    fut: Option<PendingResponse<'a, MemberBody>>,
    guild_id: GuildId,
    http: &'a Client,
    user_id: UserId,
    reason: Option<String>,
}

impl<'a> UpdateGuildMember<'a> {
    pub(crate) fn new(http: &'a Client, guild_id: GuildId, user_id: UserId) -> Self {
        Self {
            fields: UpdateGuildMemberFields::default(),
            fut: None,
            guild_id,
            http,
            user_id,
            reason: None,
        }
    }

    /// Move the member to a different voice channel.
    pub fn channel_id(mut self, channel_id: impl Into<Option<ChannelId>>) -> Self {
        self.fields.channel_id.replace(channel_id.into());

        self
    }

    /// If true, restrict the member's ability to hear sound from a voice channel.
    pub fn deaf(mut self, deaf: bool) -> Self {
        self.fields.deaf.replace(deaf);

        self
    }

    /// If true, restrict the member's ability to speak in a voice channel.
    pub fn mute(mut self, mute: bool) -> Self {
        self.fields.mute.replace(mute);

        self
    }

    /// Set the nickname.
    ///
    /// The minimum length is 1 UTF-16 character and the maximum is 32 UTF-16 characters.
    ///
    /// # Errors
    ///
    /// Returns an [`UpdateGuildMemberErrorType::NicknameInvalid`] error type if
    /// the nickname length is too short or too long.
    pub fn nick(self, nick: impl Into<Option<String>>) -> Result<Self, UpdateGuildMemberError> {
        self._nick(nick.into())
    }

    fn _nick(mut self, nick: Option<String>) -> Result<Self, UpdateGuildMemberError> {
        if let Some(nick) = nick {
            if !validate::nickname(&nick) {
                return Err(UpdateGuildMemberError {
                    kind: UpdateGuildMemberErrorType::NicknameInvalid { nickname: nick },
                });
            }

            self.fields.nick.replace(Some(nick));
        } else {
            self.fields.nick = None;
        }

        Ok(self)
    }

    /// Set the new list of roles for a member.
    pub fn roles(mut self, roles: Vec<RoleId>) -> Self {
        self.fields.roles.replace(roles);

        self
    }

    fn request(&self) -> Result<Request, HttpError> {
        let mut request = Request::builder(Route::UpdateMember {
            guild_id: self.guild_id.0,
            user_id: self.user_id.0,
        })
        .json(&self.fields)?;

        if let Some(reason) = &self.reason {
            request = request.headers(request::audit_header(reason)?);
        }

        Ok(request.build())
    }

    fn start(&mut self) -> Result<(), HttpError> {
        let request = self.request()?;
        self.fut.replace(Box::pin(self.http.request(request)));

        Ok(())
    }
}

impl<'a> AuditLogReason for UpdateGuildMember<'a> {
    fn reason(mut self, reason: impl Into<String>) -> Result<Self, AuditLogReasonError> {
        self.reason
            .replace(AuditLogReasonError::validate(reason.into())?);

        Ok(self)
    }
}

impl Future for UpdateGuildMember<'_> {
    type Output = Result<Response<MemberBody>, HttpError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            if let Some(fut) = self.as_mut().fut.as_mut() {
                return fut.as_mut().poll(cx).map_ok(|mut res| {
                    res.set_guild_id(self.guild_id);

                    res
                });
            }

            if let Err(why) = self.as_mut().start() {
                return Poll::Ready(Err(why));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{UpdateGuildMember, UpdateGuildMemberFields};
    use crate::{request::Request, routing::Route, Client};
    use std::error::Error;
    use twilight_model::id::{GuildId, UserId};

    #[test]
    fn test_request() -> Result<(), Box<dyn Error>> {
        let client = Client::new("foo");
        let guild_id = GuildId(1);
        let user_id = UserId(2);
        let builder = UpdateGuildMember::new(&client, guild_id, user_id)
            .deaf(true)
            .mute(true);
        let actual = builder.request()?;

        let body = UpdateGuildMemberFields {
            channel_id: None,
            deaf: Some(true),
            mute: Some(true),
            nick: None,
            roles: None,
        };
        let route = Route::UpdateMember {
            guild_id: guild_id.0,
            user_id: user_id.0,
        };
        let expected = Request::builder(route).json(&body)?.build();

        assert_eq!(actual.body, expected.body);
        assert_eq!(actual.path, expected.path);

        Ok(())
    }
}
