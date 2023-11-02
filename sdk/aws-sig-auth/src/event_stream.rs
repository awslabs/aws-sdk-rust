/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

// this code is dead
#![allow(deprecated)]
#![allow(clippy::disallowed_methods)]

use crate::middleware::Signature;
use aws_sigv4::event_stream::{sign_empty_message, sign_message};
use aws_sigv4::sign::v4;
use aws_smithy_eventstream::frame::{Message, SignMessage, SignMessageError};
use aws_smithy_http::property_bag::{PropertyBag, SharedPropertyBag};
use aws_smithy_runtime_api::client::identity::Identity;
use aws_types::region::SigningRegion;
use aws_types::SigningName;
use std::time::SystemTime;

/// Event Stream SigV4 signing implementation.
#[derive(Debug)]
pub struct SigV4MessageSigner {
    last_signature: String,
    identity: Identity,
    signing_region: SigningRegion,
    signing_name: SigningName,
    time: Option<SystemTime>,
}

impl SigV4MessageSigner {
    pub fn new(
        last_signature: String,
        identity: Identity,
        signing_region: SigningRegion,
        signing_name: SigningName,
        time: Option<SystemTime>,
    ) -> Self {
        Self {
            last_signature,
            identity,
            signing_region,
            signing_name,
            time,
        }
    }

    fn signing_params(&self) -> v4::SigningParams<()> {
        let builder = v4::SigningParams::builder()
            .identity(&self.identity)
            .region(self.signing_region.as_ref())
            .name(self.signing_name.as_ref())
            .time(self.time.unwrap_or_else(SystemTime::now))
            .settings(());
        builder.build().unwrap()
    }
}

impl SignMessage for SigV4MessageSigner {
    fn sign(&mut self, message: Message) -> Result<Message, SignMessageError> {
        let (signed_message, signature) = {
            let params = self.signing_params();
            sign_message(&message, &self.last_signature, &params)?.into_parts()
        };
        self.last_signature = signature;
        Ok(signed_message)
    }

    fn sign_empty(&mut self) -> Option<Result<Message, SignMessageError>> {
        let (signed_message, signature) = {
            let params = self.signing_params();
            sign_empty_message(&self.last_signature, &params)
                .expect("signing an empty message will always succeed.")
                .into_parts()
        };
        self.last_signature = signature;
        Some(Ok(signed_message))
    }
}

#[cfg(test)]
mod tests {
    use crate::event_stream::SigV4MessageSigner;
    use aws_credential_types::Credentials;
    use aws_smithy_eventstream::frame::{HeaderValue, Message, SignMessage};

    use aws_types::region::Region;
    use aws_types::region::SigningRegion;
    use aws_types::SigningName;
    use std::time::{Duration, UNIX_EPOCH};

    fn check_send_sync<T: Send + Sync>(value: T) -> T {
        value
    }

    #[test]
    fn sign_message() {
        let region = Region::new("us-east-1");
        let mut signer = check_send_sync(SigV4MessageSigner::new(
            "initial-signature".into(),
            Credentials::for_tests_with_session_token().into(),
            SigningRegion::from(region),
            SigningName::from_static("transcribe"),
            Some(UNIX_EPOCH + Duration::new(1611160427, 0)),
        ));
        let mut signatures = Vec::new();
        for _ in 0..5 {
            let signed = signer
                .sign(Message::new(&b"identical message"[..]))
                .unwrap();
            if let HeaderValue::ByteArray(signature) = signed
                .headers()
                .iter()
                .find(|h| h.name().as_str() == ":chunk-signature")
                .unwrap()
                .value()
            {
                signatures.push(signature.clone());
            } else {
                panic!("failed to get the :chunk-signature")
            }
        }
        for i in 1..signatures.len() {
            assert_ne!(signatures[i - 1], signatures[i]);
        }
    }
}

// TODO(enableNewSmithyRuntimeCleanup): Delete this old implementation that was kept around to support patch releases.
#[deprecated = "use aws_sig_auth::event_stream::SigV4MessageSigner instead (this may require upgrading the smithy-rs code generator)"]
#[derive(Debug)]
/// Event Stream SigV4 signing implementation.
pub struct SigV4Signer {
    properties: SharedPropertyBag,
    last_signature: Option<String>,
}

impl SigV4Signer {
    pub fn new(properties: SharedPropertyBag) -> Self {
        Self {
            properties,
            last_signature: None,
        }
    }

    fn signing_params(properties: &PropertyBag) -> v4::SigningParams<()> {
        // Every single one of these values would have been retrieved during the initial request,
        // so we can safely assume they all exist in the property bag at this point.
        let identity = properties.get::<Identity>().unwrap();
        let region = properties.get::<SigningRegion>().unwrap();
        let name = properties.get::<SigningName>().unwrap();
        let time = properties
            .get::<SystemTime>()
            .copied()
            .unwrap_or_else(SystemTime::now);
        let builder = v4::SigningParams::builder()
            .identity(identity)
            .region(region.as_ref())
            .name(name.as_ref())
            .time(time)
            .settings(());
        builder.build().unwrap()
    }
}

impl SignMessage for SigV4Signer {
    fn sign(&mut self, message: Message) -> Result<Message, SignMessageError> {
        let properties = self.properties.acquire();
        if self.last_signature.is_none() {
            // The Signature property should exist in the property bag for all Event Stream requests.
            self.last_signature = Some(
                properties
                    .get::<Signature>()
                    .expect("property bag contains initial Signature")
                    .as_ref()
                    .into(),
            )
        }

        let (signed_message, signature) = {
            let params = Self::signing_params(&properties);
            sign_message(&message, self.last_signature.as_ref().unwrap(), &params)?.into_parts()
        };
        self.last_signature = Some(signature);
        Ok(signed_message)
    }

    fn sign_empty(&mut self) -> Option<Result<Message, SignMessageError>> {
        let properties = self.properties.acquire();
        if self.last_signature.is_none() {
            // The Signature property should exist in the property bag for all Event Stream requests.
            self.last_signature = Some(properties.get::<Signature>().unwrap().as_ref().into())
        }
        let (signed_message, signature) = {
            let params = Self::signing_params(&properties);
            sign_empty_message(self.last_signature.as_ref().unwrap(), &params)
                .ok()?
                .into_parts()
        };
        self.last_signature = Some(signature);
        Some(Ok(signed_message))
    }
}

// TODO(enableNewSmithyRuntimeCleanup): Delete this old implementation that was kept around to support patch releases.
#[cfg(test)]
mod old_tests {
    use crate::event_stream::SigV4Signer;
    use crate::middleware::Signature;
    use aws_credential_types::Credentials;
    use aws_smithy_eventstream::frame::{HeaderValue, Message, SignMessage};
    use aws_smithy_http::property_bag::PropertyBag;
    use aws_smithy_runtime_api::client::identity::Identity;
    use aws_types::region::Region;
    use aws_types::region::SigningRegion;
    use aws_types::SigningName;
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn sign_message() {
        let region = Region::new("us-east-1");
        let mut properties = PropertyBag::new();
        properties.insert(region.clone());
        properties.insert(UNIX_EPOCH + Duration::new(1611160427, 0));
        properties.insert::<Identity>(Credentials::for_tests_with_session_token().into());
        properties.insert(SigningName::from_static("transcribe"));
        properties.insert(SigningRegion::from(region));
        properties.insert(Signature::new("initial-signature".into()));

        let mut signer = SigV4Signer::new(properties.into());
        let mut signatures = Vec::new();
        for _ in 0..5 {
            let signed = signer
                .sign(Message::new(&b"identical message"[..]))
                .unwrap();
            if let HeaderValue::ByteArray(signature) = signed
                .headers()
                .iter()
                .find(|h| h.name().as_str() == ":chunk-signature")
                .unwrap()
                .value()
            {
                signatures.push(signature.clone());
            } else {
                panic!("failed to get the :chunk-signature")
            }
        }
        for i in 1..signatures.len() {
            assert_ne!(signatures[i - 1], signatures[i]);
        }
    }
}
