use super::RequestReactionType;
use crate::{
    client::Client,
    error::Error,
    request::{PendingResponse, Request},
    response::marker::EmptyBody,
    routing::Route,
};
use twilight_model::id::{ChannelId, MessageId};

/// Delete one reaction by a user on a message.
pub struct DeleteReaction<'a> {
    channel_id: ChannelId,
    emoji: String,
    fut: Option<PendingResponse<'a, EmptyBody>>,
    http: &'a Client,
    message_id: MessageId,
    target_user: String,
}

impl<'a> DeleteReaction<'a> {
    pub(crate) fn new(
        http: &'a Client,
        channel_id: ChannelId,
        message_id: MessageId,
        emoji: RequestReactionType,
        target_user: impl Into<String>,
    ) -> Self {
        Self {
            channel_id,
            emoji: super::format_emoji(emoji),
            fut: None,
            http,
            message_id,
            target_user: target_user.into(),
        }
    }

    fn start(&mut self) -> Result<(), Error> {
        let request = Request::from_route(Route::DeleteReaction {
            channel_id: self.channel_id.0,
            emoji: self.emoji.clone(),
            message_id: self.message_id.0,
            user: self.target_user.clone(),
        });

        self.fut.replace(Box::pin(self.http.request(request)));

        Ok(())
    }
}

poll_req!(DeleteReaction<'_>, EmptyBody);
