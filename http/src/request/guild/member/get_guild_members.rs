use crate::{
    client::Client,
    error::Error as HttpError,
    request::{validate, PendingResponse, Request},
    response::{marker::MemberListBody, Response},
    routing::Route,
};
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use twilight_model::id::{GuildId, UserId};

/// The error created when the members can not be fetched as configured.
#[derive(Debug)]
pub struct GetGuildMembersError {
    kind: GetGuildMembersErrorType,
}

impl GetGuildMembersError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &GetGuildMembersErrorType {
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
        GetGuildMembersErrorType,
        Option<Box<dyn Error + Send + Sync>>,
    ) {
        (self.kind, None)
    }
}

impl Display for GetGuildMembersError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            GetGuildMembersErrorType::LimitInvalid { .. } => f.write_str("the limit is invalid"),
        }
    }
}

impl Error for GetGuildMembersError {}

/// Type of [`GetGuildMembersError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum GetGuildMembersErrorType {
    /// The limit is either 0 or more than 1000.
    LimitInvalid {
        /// Provided limit.
        limit: u64,
    },
}

#[derive(Default)]
struct GetGuildMembersFields {
    after: Option<UserId>,
    limit: Option<u64>,
    presences: Option<bool>,
}

/// Get the members of a guild, by id.
///
/// The upper limit to this request is 1000. If more than 1000 members are needed, the requests
/// must be chained. Discord defaults the limit to 1.
///
/// # Examples
///
/// Get the first 500 members of guild `100` after user ID `3000`:
///
/// ```rust,no_run
/// use twilight_http::Client;
/// use twilight_model::id::{GuildId, UserId};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
/// let client = Client::new("my token");
///
/// let guild_id = GuildId(100);
/// let user_id = UserId(3000);
/// let members = client.guild_members(guild_id).after(user_id).await?;
/// # Ok(()) }
/// ```
pub struct GetGuildMembers<'a> {
    fields: GetGuildMembersFields,
    fut: Option<PendingResponse<'a, MemberListBody>>,
    guild_id: GuildId,
    http: &'a Client,
}

impl<'a> GetGuildMembers<'a> {
    pub(crate) fn new(http: &'a Client, guild_id: GuildId) -> Self {
        Self {
            fields: GetGuildMembersFields::default(),
            fut: None,
            guild_id,
            http,
        }
    }

    /// Sets the user ID to get members after.
    pub fn after(mut self, after: UserId) -> Self {
        self.fields.after.replace(after);

        self
    }

    /// Sets the number of members to retrieve per request.
    ///
    /// The limit must be greater than 0 and less than 1000.
    ///
    /// # Errors
    ///
    /// Returns a [`GetGuildMembersErrorType::LimitInvalid`] error type if the
    /// limit is 0 or greater than 1000.
    pub fn limit(mut self, limit: u64) -> Result<Self, GetGuildMembersError> {
        if !validate::get_guild_members_limit(limit) {
            return Err(GetGuildMembersError {
                kind: GetGuildMembersErrorType::LimitInvalid { limit },
            });
        }

        self.fields.limit.replace(limit);

        Ok(self)
    }

    /// Sets whether to retrieve matched member presences
    pub fn presences(mut self, presences: bool) -> Self {
        self.fields.presences.replace(presences);

        self
    }

    fn start(&mut self) -> Result<(), HttpError> {
        let request = Request::from_route(Route::GetGuildMembers {
            after: self.fields.after.map(|x| x.0),
            guild_id: self.guild_id.0,
            limit: self.fields.limit,
            presences: self.fields.presences,
        });

        self.fut.replace(Box::pin(self.http.request(request)));

        Ok(())
    }
}

impl Future for GetGuildMembers<'_> {
    type Output = Result<Response<MemberListBody>, HttpError>;

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
