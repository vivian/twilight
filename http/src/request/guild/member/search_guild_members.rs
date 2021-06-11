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
use twilight_model::id::GuildId;

/// The error created when the members can not be queried as configured.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum SearchGuildMembersError {
    /// The limit is either 0 or more than 1000.
    LimitInvalid {
        /// Provided limit.
        limit: u64,
    },
}

impl Display for SearchGuildMembersError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::LimitInvalid { .. } => f.write_str("the limit is invalid"),
        }
    }
}

impl Error for SearchGuildMembersError {}

struct SearchGuildMembersFields {
    query: String,
    limit: Option<u64>,
}

/// Search the members of a specific guild by a query.
///
/// The upper limit to this request is 1000. Discord defaults the limit to 1.
///
/// # Examples
///
/// Get the first 10 members of guild `100` matching `Wumpus`:
///
/// ```rust,no_run
/// use twilight_http::Client;
/// use twilight_model::id::GuildId;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
/// let client = Client::new("my token");
///
/// let guild_id = GuildId(100);
/// let members = client.search_guild_members(guild_id, String::from("Wumpus")).limit(10)?.await?;
/// # Ok(()) }
/// ```
///
/// # Errors
///
/// Returns [`SearchGuildMembersError::LimitInvalid`] if the limit is invalid.
///
/// [`GUILD_MEMBERS`]: twilight_model::gateway::Intents#GUILD_MEMBERS
pub struct SearchGuildMembers<'a> {
    fields: SearchGuildMembersFields,
    fut: Option<PendingResponse<'a, MemberListBody>>,
    guild_id: GuildId,
    http: &'a Client,
}

impl<'a> SearchGuildMembers<'a> {
    pub(crate) fn new(http: &'a Client, guild_id: GuildId, query: impl Into<String>) -> Self {
        Self::_new(http, guild_id, query.into())
    }

    fn _new(http: &'a Client, guild_id: GuildId, query: String) -> Self {
        Self {
            fields: SearchGuildMembersFields { query, limit: None },
            fut: None,
            guild_id,
            http,
        }
    }

    /// Sets the number of members to retrieve per request.
    ///
    /// The limit must be greater than 0 and less than 1000.
    ///
    /// # Errors
    ///
    /// Returns [`SearchGuildMembersError::LimitInvalid`] if the limit is 0 or
    /// greater than 1000.
    pub fn limit(mut self, limit: u64) -> Result<Self, SearchGuildMembersError> {
        // Using get_guild_members_limit here as the limits are the same
        // and this endpoint is not officially documented yet.
        if !validate::search_guild_members_limit(limit) {
            return Err(SearchGuildMembersError::LimitInvalid { limit });
        }

        self.fields.limit.replace(limit);

        Ok(self)
    }

    fn start(&mut self) -> Result<(), HttpError> {
        let request = Request::from_route(Route::SearchGuildMembers {
            guild_id: self.guild_id.0,
            limit: self.fields.limit,
            query: self.fields.query.clone(),
        });

        self.fut.replace(Box::pin(self.http.request(request)));

        Ok(())
    }
}

impl Future for SearchGuildMembers<'_> {
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
