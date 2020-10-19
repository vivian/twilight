//! Structures and tools used for handling an oauth client and creating requests
//! to act on its behalf

use super::{
    authorization_url::{AuthorizationUrlBuilder, BotAuthorizationUrlBuilder},
    request::{
        access_token_exchange::AccessTokenExchangeBuilder,
        client_credentials_grant::ClientCredentialsGrantBuilder,
        refresh_token_exchange::RefreshTokenExchangeBuilder,
    },
};
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_model::id::ApplicationId;
use url::{ParseError, Url};

/// Creating a client failed due to misconfiguration.
///
/// This is returned from [`Client::new`].
///
/// [`Client::new`]: struct.Client.html#method.new
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CreateClientError<'a> {
    /// Redirect URI is not a valid URL.
    RedirectUriInvalid {
        /// Reason for the error.
        source: ParseError,
        /// Provided URI.
        uri: &'a str,
    },
}

impl Display for CreateClientError<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("creating oauth2 client failed: ")?;

        match self {
            Self::RedirectUriInvalid { source, .. } => Display::fmt(source, f),
        }
    }
}

impl Error for CreateClientError<'_> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::RedirectUriInvalid { source, .. } => Some(source),
        }
    }
}

/// Creating an authorization url failed due to an issue with the redirect uri
///
/// This is returned from any function that takes in a redirect uri
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RedirectUriInvalidError<'a> {
    /// The provided URI string isn't a valid URI.
    Invalid {
        /// Reason for the error.
        source: ParseError,
        /// Provided URI string.
        uri: &'a str,
    },
    /// The provided redirect URI is valid, but isn't in the client's list of
    /// configured redirect URIs.
    Unconfigured {
        /// Parsed URI.
        uri: Url,
    },
}

/// An oauth client
#[derive(Clone, Debug)]
pub struct Client {
    client_id: ApplicationId,
    client_secret: String,
    redirect_uris: Vec<Url>,
}

impl Client {
    /// Base URI to Discord's OAuth2 API.
    pub const BASE_URI: &'static str = "https://discord.com/api/oauth2/authorize";

    /// Create a new client with application information.
    ///
    /// # Errors
    ///
    /// Returns [`CreateClientError::RedirectUriInvalid`] if any of the provided
    /// redirect URIs are invalid URLs.
    ///
    /// [`CreateClientError::RedirectUriInvalid`]: enum.CreateClientError.html#variant.RedirectUriInvalid
    pub fn new<'a>(
        client_id: ApplicationId,
        client_secret: impl Into<String>,
        redirect_uris: &'a [&'a str],
    ) -> Result<Self, CreateClientError<'a>> {
        let iter = redirect_uris.iter();
        let mut uris = iter.size_hint().1.map_or_else(Vec::new, Vec::with_capacity);

        for item in iter {
            let uri = Url::parse(item)
                .map_err(|source| CreateClientError::RedirectUriInvalid { source, uri: item })?;

            uris.push(uri);
        }

        Ok(Self {
            client_id,
            client_secret: client_secret.into(),
            redirect_uris: uris,
        })
    }

    /// Return a builder to create a URL for bot authorization.
    ///
    /// # Examples
    ///
    /// Create a bot authorization URL requesting the "Send Messages"
    /// permission:
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use twilight_model::{guild::Permissions, id::ApplicationId};
    /// use twilight_oauth2::Client;
    ///
    /// let application_id = ApplicationId(123);
    /// let client_secret = "abcdef01234567890";
    ///
    /// let client = Client::new(application_id, client_secret, &["https://example.com"])?;
    /// let mut url_builder = client.bot_authorization_url();
    /// url_builder.permissions(Permissions::SEND_MESSAGES);
    ///
    /// println!("bot authorization url: {}", url_builder.build());
    /// # Ok(()) }
    /// ```
    pub fn bot_authorization_url(&self) -> BotAuthorizationUrlBuilder<'_> {
        BotAuthorizationUrlBuilder::new(self)
    }

    /// Create a new Authorization URL builder.
    ///
    /// The provided redirect URI is what the user will be redirected to after
    /// authorization. It must be in the client's list of configured redirect
    /// URIs.
    ///
    /// # Errors
    ///
    /// Returns [`RedirectUriInvalidError::Invalid`] if the provided redirect
    /// URI isn't a valid URL.
    ///
    /// Returns [`RedirectUriInvalidError::Unconfigured`] if the provided
    /// redirect URI isn't in the client's list of URIs.
    ///
    /// [`RedirectUriInvalidError::Invalid`]: enum.RedirectUriInvalidError.html#variant.Invalid
    /// [`RedirectUriInvalidError::Unconfigured`]: enum.RedirectUriInvalidError.html#variant.Unconfigured
    pub fn authorization_url<'a>(
        &'a self,
        redirect_uri: &'a str,
    ) -> Result<AuthorizationUrlBuilder<'a>, RedirectUriInvalidError<'a>> {
        AuthorizationUrlBuilder::new(self, redirect_uri)
    }

    /// Create an access token exchange request given a code from the initial oauth response
    ///
    /// See [Discord's example] for more information
    ///
    /// [Discord's example]: https://discord.com/developers/docs/topics/oauth2#authorization-code-grant-redirect-url-example
    pub fn access_token_exchange<'a>(&'a self, code: &'a str) -> AccessTokenExchangeBuilder<'a> {
        AccessTokenExchangeBuilder::new(self, code)
    }

    /// Create a refresh token exchange request given the user's refresh token
    ///
    /// See [Discord's documentation] for more information
    ///
    /// [Discord's documentation]: https://discord.com/developers/docs/topics/oauth2#authorization-code-grant-access-token-response
    pub fn refresh_token_exchange<'a>(
        &'a self,
        refresh_token: &'a str,
    ) -> RefreshTokenExchangeBuilder<'a> {
        RefreshTokenExchangeBuilder::new(self, refresh_token)
    }

    /// Create a client credentials grant request.
    ///
    /// A client credentials grant can be used to quickly create bearer tokens
    /// for a bot owner. Read the documentation of the builder for more
    /// information.
    pub fn client_credentials_grant(&self) -> ClientCredentialsGrantBuilder<'_> {
        ClientCredentialsGrantBuilder::new(self)
    }

    /// Return an immutable reference to the configured client ID.
    pub fn client_id(&self) -> ApplicationId {
        self.client_id
    }

    /// Return an immutable reference to the configured client secret.
    pub fn client_secret(&self) -> &str {
        self.client_secret.as_ref()
    }

    /// Return an immutable reference to the configured redirect URIs.
    pub fn redirect_uris(&self) -> &[Url] {
        self.redirect_uris.as_ref()
    }

    pub(crate) fn redirect_uri<'a>(
        &'a self,
        redirect_uri: &'a str,
    ) -> Result<&'a Url, RedirectUriInvalidError<'a>> {
        let url = Url::parse(redirect_uri).map_err(|source| RedirectUriInvalidError::Invalid {
            source,
            uri: redirect_uri,
        })?;

        self.redirect_uris()
            .iter()
            .find(|uri| **uri == url)
            .ok_or_else(|| RedirectUriInvalidError::Unconfigured { uri: url })
    }
}

#[cfg(test)]
mod tests {
    use super::{Client, CreateClientError};
    use twilight_model::id::ApplicationId;
    use url::ParseError;

    #[test]
    fn test_client_create() {
        let client = Client::new(ApplicationId(1), "a", &["https://example.com"]).unwrap();

        assert_eq!(ApplicationId(1), client.client_id());
        assert_eq!("a", client.client_secret());
        let uris = client
            .redirect_uris()
            .iter()
            .map(AsRef::as_ref)
            .collect::<Vec<_>>();
        assert_eq!(["https://example.com/"], uris.as_slice());
    }

    #[test]
    fn test_client_create_redirect_uri_invalid() {
        let actual = Client::new(ApplicationId(1), "a", &["b"]).unwrap_err();

        assert!(matches!(actual, CreateClientError::RedirectUriInvalid {
            source,
            uri,
        } if source == ParseError::RelativeUrlWithoutBase && uri == "b"));
    }
}
