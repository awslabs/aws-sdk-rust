/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_types::DateTime;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Identity {
    data: Arc<dyn Any + Send + Sync>,
    expiration: Option<DateTime>,
}

impl Identity {
    pub fn new(data: impl Any + Send + Sync, expiration: Option<DateTime>) -> Self {
        Self {
            data: Arc::new(data),
            expiration,
        }
    }

    pub fn data<T: 'static>(&self) -> Option<&T> {
        self.data.downcast_ref()
    }

    pub fn expiration(&self) -> Option<&DateTime> {
        self.expiration.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_smithy_types::date_time::Format;

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

        let expiration =
            DateTime::from_str("2023-03-15T00:00:00.000Z", Format::DateTimeWithOffset).unwrap();
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
