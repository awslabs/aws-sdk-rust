/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::client::orchestrator::Future;
use aws_smithy_http::property_bag::PropertyBag;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::SystemTime;

#[cfg(feature = "http-auth")]
pub mod http;

pub trait IdentityResolver: Send + Sync + Debug {
    fn resolve_identity(&self, identity_properties: &PropertyBag) -> Future<Identity>;
}

#[derive(Clone, Debug, Default)]
pub struct IdentityResolvers {
    identity_resolvers: Vec<(&'static str, Arc<dyn IdentityResolver>)>,
}

impl IdentityResolvers {
    pub fn builder() -> builders::IdentityResolversBuilder {
        builders::IdentityResolversBuilder::new()
    }

    pub fn identity_resolver(&self, identity_type: &'static str) -> Option<&dyn IdentityResolver> {
        self.identity_resolvers
            .iter()
            .find(|resolver| resolver.0 == identity_type)
            .map(|resolver| &*resolver.1)
    }

    pub fn to_builder(self) -> builders::IdentityResolversBuilder {
        builders::IdentityResolversBuilder {
            identity_resolvers: self.identity_resolvers,
        }
    }
}

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
    fn resolve_identity(&self, _: &PropertyBag) -> Future<Identity> {
        Future::ready(Ok(Identity::new(AnonymousIdentity::new(), None)))
    }
}

pub mod builders {
    use super::*;

    #[derive(Debug, Default)]
    pub struct IdentityResolversBuilder {
        pub(super) identity_resolvers: Vec<(&'static str, Arc<dyn IdentityResolver>)>,
    }

    impl IdentityResolversBuilder {
        pub fn new() -> Self {
            Default::default()
        }

        pub fn identity_resolver(
            mut self,
            name: &'static str,
            resolver: impl IdentityResolver + 'static,
        ) -> Self {
            self.identity_resolvers
                .push((name, Arc::new(resolver) as _));
            self
        }

        pub fn build(self) -> IdentityResolvers {
            IdentityResolvers {
                identity_resolvers: self.identity_resolvers,
            }
        }
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
