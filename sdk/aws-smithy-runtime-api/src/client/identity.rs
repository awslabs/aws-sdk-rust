/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use super::orchestrator::{BoxFallibleFut, IdentityResolver};
use aws_smithy_http::property_bag::PropertyBag;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::SystemTime;

#[derive(Clone, Debug)]
pub struct Identity {
    data: Arc<dyn Any + Send + Sync>,
    expiration: Option<SystemTime>,
}

impl Identity {
    pub fn new(data: impl Any + Send + Sync, expiration: Option<SystemTime>) -> Self {
        Self {
            data: Arc::new(data),
            expiration,
        }
    }

    pub fn data<T: 'static>(&self) -> Option<&T> {
        self.data.downcast_ref()
    }

    pub fn expiration(&self) -> Option<&SystemTime> {
        self.expiration.as_ref()
    }
}

#[derive(Debug)]
pub struct AnonymousIdentity;

impl AnonymousIdentity {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug)]
pub struct AnonymousIdentityResolver;

impl AnonymousIdentityResolver {
    pub fn new() -> Self {
        AnonymousIdentityResolver
    }
}

impl IdentityResolver for AnonymousIdentityResolver {
    fn resolve_identity(&self, _: &PropertyBag) -> BoxFallibleFut<Identity> {
        Box::pin(async { Ok(Identity::new(AnonymousIdentity::new(), None)) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_send_sync() {
        fn is_send_sync<T: Send + Sync>(_: T) {}
        is_send_sync(Identity::new("foo", None));
    }

    #[test]
    fn create_retrieve_identity() {
        #[derive(Debug)]
        struct MyIdentityData {
            first: String,
            last: String,
        }

        let expiration = SystemTime::now();
        let identity = Identity::new(
            MyIdentityData {
                first: "foo".into(),
                last: "bar".into(),
            },
            Some(expiration),
        );

        assert_eq!("foo", identity.data::<MyIdentityData>().unwrap().first);
        assert_eq!("bar", identity.data::<MyIdentityData>().unwrap().last);
        assert_eq!(Some(&expiration), identity.expiration());
    }
}
