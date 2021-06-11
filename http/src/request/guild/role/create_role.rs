use crate::{
    client::Client,
    error::Error,
    request::{self, AuditLogReason, AuditLogReasonError, PendingResponse, Request},
    routing::Route,
};
use serde::Serialize;
use twilight_model::{
    guild::{Permissions, Role},
    id::GuildId,
};

#[derive(Default, Serialize)]
struct CreateRoleFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hoist: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mentionable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    permissions: Option<Permissions>,
}

/// Create a role in a guild.
///
/// # Examples
///
/// ```rust,no_run
/// use twilight_http::Client;
/// use twilight_model::id::GuildId;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::new("my token");
/// let guild_id = GuildId(234);
///
/// client.create_role(guild_id)
///     .color(0xd90083)
///     .name("Bright Pink")
///     .await?;
/// # Ok(()) }
/// ```
pub struct CreateRole<'a> {
    fields: CreateRoleFields,
    fut: Option<PendingResponse<'a, Role>>,
    guild_id: GuildId,
    http: &'a Client,
    reason: Option<String>,
}

impl<'a> CreateRole<'a> {
    pub(crate) fn new(http: &'a Client, guild_id: GuildId) -> Self {
        Self {
            fields: CreateRoleFields::default(),
            fut: None,
            guild_id,
            http,
            reason: None,
        }
    }

    /// Set the color of the role.
    pub fn color(mut self, color: u32) -> Self {
        self.fields.color.replace(color);

        self
    }

    /// If true, display the role in the members list.
    pub fn hoist(mut self, hoist: bool) -> Self {
        self.fields.hoist.replace(hoist);

        self
    }

    /// If true, the role can be @mentioned (pinged) in chat.
    pub fn mentionable(mut self, mentionable: bool) -> Self {
        self.fields.mentionable.replace(mentionable);

        self
    }

    /// Set the name of the role.
    ///
    /// If none is specified, Discord sets this to `New Role`.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.fields.name.replace(name.into());

        self
    }

    /// Set the allowed permissions of this role.
    pub fn permissions(mut self, permissions: Permissions) -> Self {
        self.fields.permissions.replace(permissions);

        self
    }

    fn start(&mut self) -> Result<(), Error> {
        let mut request = Request::builder(Route::CreateRole {
            guild_id: self.guild_id.0,
        })
        .json(&self.fields)?;

        if let Some(reason) = &self.reason {
            request = request.headers(request::audit_header(reason)?);
        }

        self.fut
            .replace(Box::pin(self.http.request(request.build())));

        Ok(())
    }
}

impl<'a> AuditLogReason for CreateRole<'a> {
    fn reason(mut self, reason: impl Into<String>) -> Result<Self, AuditLogReasonError> {
        self.reason
            .replace(AuditLogReasonError::validate(reason.into())?);

        Ok(self)
    }
}

poll_req!(CreateRole<'_>, Role);
