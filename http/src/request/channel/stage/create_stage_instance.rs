use crate::{
    client::Client,
    error::Error as HttpError,
    request::{validate, PendingResponse, Request},
    response::marker::EmptyBody,
    routing::Route,
};
use serde::Serialize;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_model::{channel::stage_instance::PrivacyLevel, id::ChannelId};

/// The request can not be created as configured.
#[derive(Debug)]
pub struct CreateStageInstanceError {
    kind: CreateStageInstanceErrorType,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl CreateStageInstanceError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &CreateStageInstanceErrorType {
        &self.kind
    }

    /// Consume the error, returning the source error if there is any.
    #[must_use = "consuming the error and retrieving the source has no effect if left unused"]
    pub fn into_source(self) -> Option<Box<dyn Error + Send + Sync>> {
        self.source
    }

    /// Consume the error, returning the owned error type and the source error.
    #[must_use = "consuming the error into its parts has no effect if left unused"]
    pub fn into_parts(
        self,
    ) -> (
        CreateStageInstanceErrorType,
        Option<Box<dyn Error + Send + Sync>>,
    ) {
        (self.kind, self.source)
    }
}

impl Display for CreateStageInstanceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            CreateStageInstanceErrorType::InvalidTopic { .. } => f.write_str("invalid topic"),
        }
    }
}

impl Error for CreateStageInstanceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| &**source as &(dyn Error + 'static))
    }
}

#[derive(Debug)]
pub enum CreateStageInstanceErrorType {
    /// Topic is not between 1 and 120 characters in length.
    InvalidTopic {
        /// Invalid topic.
        topic: String,
    },
}

#[derive(Default, Serialize)]
struct CreateStageInstanceFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    privacy_level: Option<PrivacyLevel>,
}

/// Create a new stage instance associated with a stage channel.
///
/// Requires the user to be a moderator of the stage channel.
pub struct CreateStageInstance<'a> {
    channel_id: ChannelId,
    fields: CreateStageInstanceFields,
    fut: Option<PendingResponse<'a, EmptyBody>>,
    http: &'a Client,
    topic: String,
}

impl<'a> CreateStageInstance<'a> {
    pub(crate) fn new(
        http: &'a Client,
        channel_id: ChannelId,
        topic: impl Into<String>,
    ) -> Result<Self, CreateStageInstanceError> {
        Self::_new(http, channel_id, topic.into())
    }

    fn _new(
        http: &'a Client,
        channel_id: ChannelId,
        topic: String,
    ) -> Result<Self, CreateStageInstanceError> {
        if !validate::stage_topic(&topic) {
            return Err(CreateStageInstanceError {
                kind: CreateStageInstanceErrorType::InvalidTopic { topic },
                source: None,
            });
        }

        Ok(Self {
            channel_id,
            fields: CreateStageInstanceFields::default(),
            fut: None,
            http,
            topic,
        })
    }

    /// Set the [`PrivacyLevel`] of the instance.
    pub fn privacy_level(mut self, privacy_level: PrivacyLevel) -> Self {
        self.fields.privacy_level.replace(privacy_level);

        self
    }

    fn start(&mut self) -> Result<(), HttpError> {
        let request = Request::builder(Route::CreateStageInstance {
            channel_id: self.channel_id.0,
            topic: self.topic.clone(),
        })
        .json(&self.fields)?
        .build();

        self.fut.replace(Box::pin(self.http.request(request)));

        Ok(())
    }
}

poll_req!(CreateStageInstance<'_>, EmptyBody);
