use crate::{
    client::Client,
    error::Error as HttpError,
    request::{self, validate, AuditLogReason, AuditLogReasonError, PendingResponse, Request},
    routing::Route,
};
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_model::{
    guild::GuildPrune,
    id::{GuildId, RoleId},
};

/// The error created when the guild prune can not be created as configured.
#[derive(Debug)]
pub struct CreateGuildPruneError {
    kind: CreateGuildPruneErrorType,
}

impl CreateGuildPruneError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &CreateGuildPruneErrorType {
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
        CreateGuildPruneErrorType,
        Option<Box<dyn Error + Send + Sync>>,
    ) {
        (self.kind, None)
    }
}

impl Display for CreateGuildPruneError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            CreateGuildPruneErrorType::DaysInvalid { .. } => {
                f.write_str("the number of days is invalid")
            }
        }
    }
}

impl Error for CreateGuildPruneError {}

/// Type of [`CreateGuildPruneError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum CreateGuildPruneErrorType {
    /// The number of days is 0.
    DaysInvalid,
}

#[derive(Default)]
struct CreateGuildPruneFields {
    compute_prune_count: Option<bool>,
    days: Option<u64>,
    include_roles: Vec<u64>,
}

/// Begin a guild prune.
///
/// Refer to [the discord docs] for more information.
///
/// [the discord docs]: https://discord.com/developers/docs/resources/guild#begin-guild-prune
pub struct CreateGuildPrune<'a> {
    fields: CreateGuildPruneFields,
    guild_id: GuildId,
    fut: Option<PendingResponse<'a, Option<GuildPrune>>>,
    http: &'a Client,
    reason: Option<String>,
}

impl<'a> CreateGuildPrune<'a> {
    pub(crate) fn new(http: &'a Client, guild_id: GuildId) -> Self {
        Self {
            fields: CreateGuildPruneFields::default(),
            fut: None,
            guild_id,
            http,
            reason: None,
        }
    }

    /// List of roles to include when pruning.
    pub fn include_roles(mut self, roles: impl Iterator<Item = RoleId>) -> Self {
        let roles = roles.map(|e| e.0).collect::<Vec<_>>();

        self.fields.include_roles = roles;

        self
    }

    /// Return the amount of pruned members. Discouraged for large guilds.
    pub fn compute_prune_count(mut self, compute_prune_count: bool) -> Self {
        self.fields.compute_prune_count.replace(compute_prune_count);

        self
    }

    /// Set the number of days that a user must be inactive before being pruned.
    ///
    /// The number of days must be greater than 0.
    ///
    /// # Errors
    ///
    /// Returns a [`CreateGuildPruneErrorType::DaysInvalid`] error type if the
    /// number of days is 0.
    pub fn days(mut self, days: u64) -> Result<Self, CreateGuildPruneError> {
        if !validate::guild_prune_days(days) {
            return Err(CreateGuildPruneError {
                kind: CreateGuildPruneErrorType::DaysInvalid,
            });
        }

        self.fields.days.replace(days);

        Ok(self)
    }

    fn start(&mut self) -> Result<(), HttpError> {
        let mut request = Request::builder(Route::CreateGuildPrune {
            compute_prune_count: self.fields.compute_prune_count,
            days: self.fields.days,
            guild_id: self.guild_id.0,
            include_roles: self.fields.include_roles.clone(),
        });

        if let Some(reason) = self.reason.as_ref() {
            request = request.headers(request::audit_header(reason)?);
        }

        self.fut
            .replace(Box::pin(self.http.request(request.build())));

        Ok(())
    }
}

impl<'a> AuditLogReason for CreateGuildPrune<'a> {
    fn reason(mut self, reason: impl Into<String>) -> Result<Self, AuditLogReasonError> {
        self.reason
            .replace(AuditLogReasonError::validate(reason.into())?);

        Ok(self)
    }
}

poll_req!(CreateGuildPrune<'_>, Option<GuildPrune>);
