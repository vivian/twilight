use crate::{
    client::Client,
    error::Error,
    request::{self, AuditLogReason, AuditLogReasonError, PendingResponse, Request},
    response::marker::EmptyBody,
    routing::Route,
};
use twilight_model::id::{GuildId, RoleId, UserId};

/// Add a role to a member in a guild.
///
/// # Examples
///
/// In guild `1`, add role `2` to user `3`, for the reason `"test"`:
///
/// ```rust,no_run
/// use twilight_http::{request::AuditLogReason, Client};
/// use twilight_model::id::{GuildId, RoleId, UserId};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
/// let client = Client::new("my token");
///
/// let guild_id = GuildId(1);
/// let role_id = RoleId(2);
/// let user_id = UserId(3);
///
/// client.add_guild_member_role(guild_id, user_id, role_id).reason("test")?.await?;
/// # Ok(()) }
/// ```
pub struct AddRoleToMember<'a> {
    fut: Option<PendingResponse<'a, EmptyBody>>,
    guild_id: GuildId,
    http: &'a Client,
    role_id: RoleId,
    user_id: UserId,
    reason: Option<String>,
}

impl<'a> AddRoleToMember<'a> {
    pub(crate) fn new(
        http: &'a Client,
        guild_id: impl Into<GuildId>,
        user_id: impl Into<UserId>,
        role_id: impl Into<RoleId>,
    ) -> Self {
        Self {
            fut: None,
            guild_id: guild_id.into(),
            http,
            role_id: role_id.into(),
            user_id: user_id.into(),
            reason: None,
        }
    }

    fn start(&mut self) -> Result<(), Error> {
        let mut request = Request::builder(Route::AddMemberRole {
            guild_id: self.guild_id.0,
            role_id: self.role_id.0,
            user_id: self.user_id.0,
        });

        if let Some(reason) = self.reason.as_ref() {
            request = request.headers(request::audit_header(reason)?);
        }

        self.fut
            .replace(Box::pin(self.http.request(request.build())));

        Ok(())
    }
}

impl<'a> AuditLogReason for AddRoleToMember<'a> {
    fn reason(mut self, reason: impl Into<String>) -> Result<Self, AuditLogReasonError> {
        self.reason
            .replace(AuditLogReasonError::validate(reason.into())?);

        Ok(self)
    }
}

poll_req!(AddRoleToMember<'_>, EmptyBody);
