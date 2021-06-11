use super::execute_webhook::ExecuteWebhookFields;
use crate::{
    client::Client,
    error::Error,
    request::{PendingResponse, Request},
    routing::Route,
};
use twilight_model::{channel::Message, id::WebhookId};

/// Execute a webhook, sending a message to its channel, and then wait for the
/// message to be created.
///
/// # Examples
///
/// ```rust,no_run
/// use twilight_http::Client;
/// use twilight_model::id::WebhookId;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::new("my token");
/// let id = WebhookId(432);
///
/// let message = client
///     .execute_webhook(id, "webhook token")
///     .content("Pinkie...")
///     .wait()
///     .await?
///     .model()
///     .await?;
///
/// println!("message id: {}", message.id);
/// # Ok(()) }
/// ```
///
/// [`content`]: Self::content
/// [`embeds`]: Self::embeds
/// [`file`]: Self::file
pub struct ExecuteWebhookAndWait<'a> {
    pub(crate) fields: ExecuteWebhookFields,
    fut: Option<PendingResponse<'a, Message>>,
    http: &'a Client,
    token: String,
    webhook_id: WebhookId,
}

impl<'a> ExecuteWebhookAndWait<'a> {
    pub(crate) fn new(
        http: &'a Client,
        webhook_id: WebhookId,
        token: String,
        fields: ExecuteWebhookFields,
    ) -> Self {
        Self {
            fields,
            fut: None,
            http,
            token,
            webhook_id,
        }
    }

    fn start(&mut self) -> Result<(), Error> {
        let request = Request::from((
            crate::json::to_vec(&self.fields).map_err(Error::json)?,
            Route::ExecuteWebhook {
                token: self.token.clone(),
                wait: Some(true),
                webhook_id: self.webhook_id.0,
            },
        ));

        self.fut.replace(Box::pin(self.http.request(request)));

        Ok(())
    }
}

poll_req!(ExecuteWebhookAndWait<'_>, Message);
