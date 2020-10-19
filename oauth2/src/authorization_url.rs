//! The structures and tools needed to construct and deal with authorization urls

use super::{
    client::{Client, RedirectUriInvalidError},
    Prompt, Scope,
};
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use twilight_model::{guild::Permissions, id::GuildId};
use url::Url;

/// Type of response to give after authorization approval.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum ResponseType {
    /// Provide a code response.
    ///
    /// This will contain an authorization code in the query parameters of the
    /// URI that the user is redirected to.
    Code,
    /// Provide a token response.
    ///
    /// This will contain an access token in the query fragment of the URI that
    /// the user is redirected to.
    Token,
}

impl ResponseType {
    /// Name of the response type.
    ///
    /// # Examples
    ///
    /// ```
    /// use twilight_oauth2::authorization_url::ResponseType;
    ///
    /// assert_eq!("code", ResponseType::Code.name());
    /// assert_eq!("token", ResponseType::Token.name());
    /// ```
    pub fn name(self) -> &'static str {
        match self {
            Self::Code => "code",
            Self::Token => "token",
        }
    }
}

/// A builder to construct an authorization url
pub struct AuthorizationUrlBuilder<'a> {
    client: &'a Client,
    prompt: Option<Prompt>,
    redirect_uri: &'a Url,
    scopes: Option<&'a [Scope]>,
    state: Option<&'a str>,
}

impl<'a> AuthorizationUrlBuilder<'a> {
    pub(crate) fn new(
        client: &'a Client,
        redirect_uri: &'a str,
    ) -> Result<Self, RedirectUriInvalidError<'a>> {
        let redirect_uri = client.redirect_uri(redirect_uri)?;

        Ok(Self {
            client,
            prompt: None,
            redirect_uri,
            scopes: None,
            state: None,
        })
    }

    /// Build the authorization URL into a code grant URL
    ///
    /// This is the standard oauth method requiring a handshake between the
    /// server and client after the code had been granted. Please refer to the
    /// [Discord documentation] for more information.
    ///
    /// For implicit grant, see [`implicit_grant`]
    ///
    /// [`implicit_grant`]: #method.implicit_grant
    /// [Discord documentation]: https://discord.com/developers/docs/topics/oauth2#authorization-code-grant
    pub fn build(&self) -> String {
        self.build_with_response_type(ResponseType::Code)
    }

    /// Build the authorization URL into an implicit grant URL.
    ///
    /// Contrasted from [`build`], this will contain URI fragments after a hash
    /// (`#`) instead of query parameters. Please refer to the
    /// [Discord documentation] for more information.
    ///
    /// [`build`]: #method.build
    /// [Discord documentation]: https://discord.com/developers/docs/topics/oauth2#implicit-grant
    pub fn implicit_grant(&self) -> String {
        self.build_with_response_type(ResponseType::Token)
    }

    /// Build the authorization URL as a webhook authorization.
    ///
    /// This will set the scope to [`WebhookIncoming`], and can be used to have
    /// the user create and authorize a webhook. Refer to
    /// [`webhook_token_exchange`] for more information.
    ///
    /// [`WebhookIncoming`]: ../enum.Scope.html#variant.WebhookIncoming
    /// [`webhook_token_exchange`]: ../request/webhook_token_exchange/index.html
    pub fn webhook(&mut self) -> String {
        self.scopes(&[Scope::WebhookIncoming]);

        self.build_with_response_type(ResponseType::Code)
    }

    fn build_with_response_type(&self, response_type: ResponseType) -> String {
        let mut url = Client::BASE_URI.to_owned();
        url.push('?');
        url.push_str("response_type=");
        url.push_str(response_type.name());
        url.push_str("&client_id=");
        write!(url, "{}", self.client.client_id().0).expect("client id write can't error");

        if let Some(scopes) = self.scopes.as_ref() {
            url.push_str("&scope=");

            let scope_count = scopes.len().saturating_sub(1);

            for (idx, scope) in scopes.iter().enumerate() {
                url.push_str(scope.name());

                if idx < scope_count {
                    url.push_str("%20");
                }
            }
        }

        if let Some(state) = self.state.as_ref() {
            url.push_str("&state=");
            url.push_str(state);
        }

        url.push_str("&redirect_uri=");
        url.push_str(&urlencoding::encode(self.redirect_uri.as_ref()));

        if let Some(prompt) = self.prompt.as_ref().map(Prompt::name) {
            url.push_str("&prompt=");
            url.push_str(prompt);
        }

        url
    }

    /// Set how to prompt the user for authorization.
    ///
    /// Read the documentation for [`Prompt`] for information on what meaning
    /// each variant has.
    ///
    /// For the [`Scope::Bot`] and [`Scope::WebhookIncoming`] scopes the prompt
    /// will always be [`Prompt::Consent`]. Defaults to Discord's default.
    ///
    /// [`Prompt`]: enum.Prompt.html
    /// [`Scope::Bot`]: enum.Scope.html#variant.Bot
    /// [`Scope::WebhookIncoming`]: enum.Scope.html#variant.WebhookIncoming
    pub fn prompt(&mut self, prompt: Prompt) -> &mut Self {
        self.prompt.replace(prompt);

        self
    }

    /// Set the scopes for the authorization request.
    ///
    /// If you set the [`WebhookIncoming`] scope then a webhook will be
    /// returned on the [`AccessTokenExchangeResponse`].
    ///
    /// Read about Discord's [scope documentation].
    ///
    /// [RFC 6749 § 3.3] on access token scopes.
    ///
    /// [`AccessTokenExchangeResponse`]: ../request/access_token_exchange/struct.AccessTokenExchangeResponse.html
    /// [`WebhookIncoming`]: ../enum.Scope.html#variant.WebhookIncoming
    /// [RFC 6749 § 3.3]: https://tools.ietf.org/html/rfc6749#section-3.3
    /// [scope documentation]: https://discord.com/developers/docs/topics/oauth2#shared-resources-oauth2-scopes
    pub fn scopes(&mut self, scopes: &'a [Scope]) -> &mut Self {
        self.scopes.replace(scopes);

        self
    }

    /// Set the state for the authorization request.
    ///
    /// Read about Discord's recommendations for [state and security] for more
    /// information.
    ///
    /// [RFC 6749 § 4.1.1] on access token scopes.
    ///
    /// [RFC 6749 § 4.1.1]: https://tools.ietf.org/html/rfc6749#section-4.1.1
    /// [state and security]: https://discord.com/developers/docs/topics/oauth2#state-and-security
    pub fn state(&mut self, state: &'a str) -> &mut Self {
        self.state.replace(state);

        self
    }
}

/// A builder to construct a bot authorization url.
///
/// See [Discord's Bot Authorization Flow] for more info
///
/// [Discord's Bot Authorization Flow]: https://discord.com/developers/docs/topics/oauth2#bot-authorization-flow
pub struct BotAuthorizationUrlBuilder<'a> {
    client: &'a Client,
    disable_guild_select: Option<bool>,
    guild_id: Option<GuildId>,
    permissions: Option<Permissions>,
    redirect_uri: Option<&'a Url>,
    scopes: &'a [Scope],
}

impl<'a> BotAuthorizationUrlBuilder<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            disable_guild_select: None,
            guild_id: None,
            permissions: None,
            redirect_uri: None,
            scopes: &[Scope::Bot],
        }
    }

    /// Build a bot authorization URL.
    pub fn build(&self) -> String {
        let mut url = Client::BASE_URI.to_owned();
        url.push_str("?client_id=");
        let _ = write!(url, "{}", self.client.client_id().0);

        if let Some(disable_guild_select) = self.disable_guild_select {
            url.push_str("&disable_guild_select=");
            url.push_str(if disable_guild_select {
                "true"
            } else {
                "false"
            });
        }

        if let Some(guild_id) = self.guild_id {
            url.push_str("&guild_id=");
            let _ = write!(url, "{}", guild_id.0);
        }

        if let Some(permissions) = self.permissions {
            url.push_str("&permissions=");
            let _ = write!(url, "{}", permissions.bits());
        }

        if let Some(redirect_uri) = self.redirect_uri.as_ref() {
            url.push_str("&redirect_uri=");
            url.push_str(&urlencoding::encode(redirect_uri.as_ref()));
        }

        url.push_str("&scope=");

        let scope_count = self.scopes.len().saturating_sub(1);

        for (idx, scope) in self.scopes.iter().enumerate() {
            url.push_str(scope.name());

            if idx < scope_count {
                url.push_str("%20");
            }
        }

        url
    }

    /// Set whether to disable selection of what guild to authorize.
    ///
    /// If you set this to `true`, then you must set [`guild_id`].
    ///
    /// [`guild_id`]: #method.guild_id
    pub fn disable_guild_select(&mut self, disable_guild_select: bool) -> &mut Self {
        self.disable_guild_select.replace(disable_guild_select);

        self
    }

    /// Set the ID of the guild to recommend the user to authorize the bot to.
    ///
    /// You must set this if [`disable_guild_select`] is set to `true`.
    ///
    /// [`disable_guild_select`]: #method.disable_guild_select
    pub fn guild_id(&mut self, guild_id: GuildId) -> &mut Self {
        self.guild_id.replace(guild_id);

        self
    }

    /// Set the permissions to request the bot be given when authorized to a
    /// guild.
    pub fn permissions(&mut self, permissions: Permissions) -> &mut Self {
        self.permissions.replace(permissions);

        self
    }

    /// Set the Redirect URI to redirect the user to.
    ///
    /// This will only be used if you [configure scopes] other than the
    /// [`Bot`] scope.
    ///
    /// # Errors
    ///
    /// Returns [`RedirectUriInvalidError::Invalid`] if the provided redirect
    /// URI isn't a valid URL.
    ///
    /// Returns [`RedirectUriInvalidError::Unconfigured`] if the provided
    /// redirect URI isn't in the client's list of URIs.
    ///
    /// [`Bot`]: enum.Scope.html#variant.Bot
    /// [`RedirectUriInvalidError::Invalid`]: ../client/enum.RedirectUriInvalidError.html#variant.Invalid
    /// [`RedirectUriInvalidError::Unconfigured`]: ../client/enum.RedirectUriInvalidError.html#variant.Unconfigured
    /// [configure scopes]: #method.scopes
    pub fn redirect_uri(
        &mut self,
        redirect_uri: &'a str,
    ) -> Result<&mut Self, RedirectUriInvalidError<'_>> {
        let url = self.client.redirect_uri(redirect_uri)?;

        self.redirect_uri.replace(url);

        Ok(self)
    }

    /// Set the scopes for the bot authorization request.
    ///
    /// By default the [`Bot`] scope is selected.
    ///
    /// Read about Discord's [scope documentation].
    ///
    /// [RFC 6749 § 3.3] on access token scopes.
    ///
    /// [`Bot`]: enum.Scope.html#variant.Bot
    /// [RFC 6749 § 3.3]: https://tools.ietf.org/html/rfc6749#section-3.3
    /// [scope documentation]: https://discord.com/developers/docs/topics/oauth2#shared-resources-oauth2-scopes
    pub fn scopes(&mut self, scopes: &'a [Scope]) -> &mut Self {
        self.scopes = scopes;

        self
    }
}

#[cfg(test)]
mod tests {
    use super::{Client, Scope};
    use twilight_model::{
        guild::Permissions,
        id::{ApplicationId, GuildId},
    };

    #[test]
    fn test_bot_authorization_url() {
        let client = Client::new(ApplicationId(1), "a", &["https://example.com/"]).unwrap();
        let mut builder = client.bot_authorization_url();
        let expected = "https://discord.com/api/oauth2/authorize?client_id=1&scope=bot";
        assert_eq!(expected, builder.build());

        let perms = Permissions::SEND_MESSAGES | Permissions::MANAGE_MESSAGES;
        builder.permissions(perms);
        let expected = format!(
            "https://discord.com/api/oauth2/authorize?\
            client_id=1\
            &permissions={}\
            &scope=bot",
            perms.bits()
        );
        assert_eq!(expected, builder.build());

        builder.guild_id(GuildId(2));
        let expected = format!(
            "https://discord.com/api/oauth2/authorize?\
            client_id=1\
            &guild_id=2\
            &permissions={}\
            &scope=bot",
            perms.bits()
        );
        assert_eq!(expected, builder.build());

        builder.disable_guild_select(true);
        let expected = format!(
            "https://discord.com/api/oauth2/authorize?\
            client_id=1\
            &disable_guild_select=true\
            &guild_id=2\
            &permissions={}\
            &scope=bot",
            perms.bits()
        );
        assert_eq!(expected, builder.build());

        builder.redirect_uri("https://example.com").unwrap();
        let expected = format!(
            "https://discord.com/api/oauth2/authorize?\
            client_id=1\
            &disable_guild_select=true\
            &guild_id=2\
            &permissions={}\
            &redirect_uri=https%3A%2F%2Fexample.com%2F\
            &scope=bot",
            perms.bits()
        );
        assert_eq!(expected, builder.build());

        builder.scopes(&[Scope::Bot, Scope::GuildsJoin]);
        let expected = format!(
            "https://discord.com/api/oauth2/authorize?\
            client_id=1\
            &disable_guild_select=true\
            &guild_id=2\
            &permissions={}\
            &redirect_uri=https%3A%2F%2Fexample.com%2F\
            &scope=bot%20guilds.join",
            perms.bits()
        );
        assert_eq!(expected, builder.build());
    }

    #[test]
    fn test_implicit_grant() {
        let client = Client::new(ApplicationId(1), "a", &["https://example.com/"]).unwrap();
        let builder = client.authorization_url("https://example.com/").unwrap();
        let expected = "https://discord.com/api/oauth2/authorize?\
            response_type=token\
            &client_id=1\
            &redirect_uri=https%3A%2F%2Fexample.com%2F";
        assert_eq!(expected, builder.implicit_grant());
    }

    #[test]
    fn test_webhook() {
        let client = Client::new(ApplicationId(1), "a", &["https://example.com/"]).unwrap();
        let mut builder = client.authorization_url("https://example.com/").unwrap();
        let expected = "https://discord.com/api/oauth2/authorize?\
            response_type=code\
            &client_id=1\
            &scope=webhook.incoming\
            &redirect_uri=https%3A%2F%2Fexample.com%2F";
        assert_eq!(expected, builder.webhook());
    }
}
