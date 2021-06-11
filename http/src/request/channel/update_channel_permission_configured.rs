use crate::{
    client::Client,
    error::Error,
    request::{self, AuditLogReason, AuditLogReasonError, PendingResponse, Request},
    response::marker::EmptyBody,
    routing::Route,
};
use serde::Serialize;
use twilight_model::{
    channel::permission_overwrite::{PermissionOverwriteTargetType, PermissionOverwriteType},
    guild::Permissions,
    id::ChannelId,
};

#[derive(Serialize)]
struct UpdateChannelPermissionConfiguredFields {
    allow: Permissions,
    deny: Permissions,
    #[serde(rename = "type")]
    kind: PermissionOverwriteTargetType,
}

/// Created when either `member` or `role` is called on a `DeleteChannelPermission` struct.
pub struct UpdateChannelPermissionConfigured<'a> {
    channel_id: ChannelId,
    fields: UpdateChannelPermissionConfiguredFields,
    fut: Option<PendingResponse<'a, EmptyBody>>,
    http: &'a Client,
    target_id: u64,
    reason: Option<String>,
}

impl<'a> UpdateChannelPermissionConfigured<'a> {
    pub(crate) fn new(
        http: &'a Client,
        channel_id: ChannelId,
        allow: Permissions,
        deny: Permissions,
        target: &PermissionOverwriteType,
    ) -> Self {
        let (name, target_id) = match target {
            PermissionOverwriteType::Member(user_id) => {
                (PermissionOverwriteTargetType::Member, user_id.0)
            }
            PermissionOverwriteType::Role(role_id) => {
                (PermissionOverwriteTargetType::Role, role_id.0)
            }
        };

        Self {
            channel_id,
            fields: UpdateChannelPermissionConfiguredFields {
                allow,
                deny,
                kind: name,
            },
            fut: None,
            http,
            target_id,
            reason: None,
        }
    }

    fn request(&self) -> Result<Request, Error> {
        let mut request = Request::builder(Route::UpdatePermissionOverwrite {
            channel_id: self.channel_id.0,
            target_id: self.target_id,
        })
        .json(&self.fields)?;

        if let Some(reason) = &self.reason {
            request = request.headers(request::audit_header(reason)?);
        }

        Ok(request.build())
    }

    fn start(&mut self) -> Result<(), Error> {
        let request = self.request()?;

        self.fut.replace(Box::pin(self.http.request(request)));

        Ok(())
    }
}

impl<'a> AuditLogReason for UpdateChannelPermissionConfigured<'a> {
    fn reason(mut self, reason: impl Into<String>) -> Result<Self, AuditLogReasonError> {
        self.reason
            .replace(AuditLogReasonError::validate(reason.into())?);

        Ok(self)
    }
}

poll_req!(UpdateChannelPermissionConfigured<'_>, EmptyBody);

#[cfg(test)]
mod tests {
    use super::{UpdateChannelPermissionConfigured, UpdateChannelPermissionConfiguredFields};
    use crate::{request::Request, routing::Route, Client};
    use twilight_model::{
        channel::permission_overwrite::{PermissionOverwriteTargetType, PermissionOverwriteType},
        guild::Permissions,
        id::{ChannelId, UserId},
    };

    #[test]
    fn test_request() {
        let client = Client::new("foo");
        let builder = UpdateChannelPermissionConfigured::new(
            &client,
            ChannelId(1),
            Permissions::empty(),
            Permissions::SEND_MESSAGES,
            &PermissionOverwriteType::Member(UserId(2)),
        );
        let actual = builder.request().expect("failed to create request");

        let body = crate::json::to_vec(&UpdateChannelPermissionConfiguredFields {
            allow: Permissions::empty(),
            deny: Permissions::SEND_MESSAGES,
            kind: PermissionOverwriteTargetType::Member,
        })
        .expect("failed to serialize payload");
        let route = Route::UpdatePermissionOverwrite {
            channel_id: 1,
            target_id: 2,
        };
        let expected = Request::builder(route).body(body).build();

        assert_eq!(expected.body, actual.body);
        assert_eq!(expected.path, actual.path);
    }
}
