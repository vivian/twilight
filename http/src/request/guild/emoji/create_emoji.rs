use crate::{
    client::Client,
    error::Error,
    request::{self, AuditLogReason, AuditLogReasonError, PendingResponse, Request},
    routing::Route,
};
use serde::Serialize;
use twilight_model::{
    guild::Emoji,
    id::{GuildId, RoleId},
};

#[derive(Serialize)]
struct CreateEmojiFields {
    image: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    roles: Option<Vec<RoleId>>,
}

/// Create an emoji in a guild.
///
/// The emoji must be a Data URI, in the form of `data:image/{type};base64,{data}` where `{type}`
/// is the image MIME type and `{data}` is the base64-encoded image.  Refer to [the discord docs]
/// for more information about image data.
///
/// [the discord docs]: https://discord.com/developers/docs/reference#image-data
pub struct CreateEmoji<'a> {
    fut: Option<PendingResponse<'a, Emoji>>,
    fields: CreateEmojiFields,
    guild_id: GuildId,
    http: &'a Client,
    reason: Option<String>,
}

impl<'a> CreateEmoji<'a> {
    pub(crate) fn new(
        http: &'a Client,
        guild_id: GuildId,
        name: impl Into<String>,
        image: impl Into<String>,
    ) -> Self {
        Self {
            fields: CreateEmojiFields {
                image: image.into(),
                name: name.into(),
                roles: None,
            },
            fut: None,
            guild_id,
            http,
            reason: None,
        }
    }

    /// Whitelist roles for this emoji.
    ///
    /// Refer to [the discord docs] for more information.
    ///
    /// [the discord docs]: https://discord.com/developers/docs/resources/emoji
    pub fn roles(mut self, roles: Vec<RoleId>) -> Self {
        self.fields.roles.replace(roles);

        self
    }

    fn start(&mut self) -> Result<(), Error> {
        let mut request = Request::builder(Route::CreateEmoji {
            guild_id: self.guild_id.0,
        })
        .json(&self.fields)?;

        if let Some(reason) = self.reason.as_ref() {
            request = request.headers(request::audit_header(reason)?);
        }

        self.fut
            .replace(Box::pin(self.http.request(request.build())));

        Ok(())
    }
}

impl<'a> AuditLogReason for CreateEmoji<'a> {
    fn reason(mut self, reason: impl Into<String>) -> Result<Self, AuditLogReasonError> {
        self.reason
            .replace(AuditLogReasonError::validate(reason.into())?);

        Ok(self)
    }
}

poll_req!(CreateEmoji<'_>, Emoji);
