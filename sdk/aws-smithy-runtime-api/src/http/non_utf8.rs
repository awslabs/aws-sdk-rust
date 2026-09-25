/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Handling for response header values that are not valid UTF-8

use aws_smithy_types::config_bag::{Storable, StoreReplace};

/// What to do when a response header value bound to a modeled member is not valid UTF-8
///
/// An HTTP header value may contain any octet in `0x80..=0xFF` (obs-text, RFC 7230), and an
/// arbitrary sequence of those is not necessarily valid UTF-8, so a service may send a value that
/// cannot be represented as a Rust `String`. [`Headers`](crate::http::Headers) stores
/// such a value as received, but the modeled member it is bound to is a `String`, so something has
/// to give when the member is deserialized.
///
/// The default is [`Reject`](Self::Reject). To choose otherwise, put this in the config bag from an
/// interceptor that runs before deserialization. [`Skip`](Self::Skip) does not remove the header, so
/// the same interceptor can read the octets from any hook that sees the response, using
/// [`Headers::get_bytes`](crate::http::Headers::get_bytes) or
/// [`iter_bytes`](crate::http::Headers::iter_bytes):
///
#[cfg_attr(
    feature = "client",
    doc = r#"
```no_run
# use aws_smithy_runtime_api::box_error::BoxError;
# use aws_smithy_runtime_api::client::interceptors::context::{
#     BeforeDeserializationInterceptorContextRef, BeforeSerializationInterceptorContextRef,
# };
# use aws_smithy_runtime_api::client::interceptors::Intercept;
# use aws_smithy_runtime_api::client::runtime_components::RuntimeComponents;
# use aws_smithy_runtime_api::http::NonUtf8HeaderHandling;
# use aws_smithy_types::config_bag::ConfigBag;
# use std::sync::{Arc, Mutex};
#[derive(Clone, Debug, Default)]
struct SkipNonUtf8Headers {
    seen: Arc<Mutex<Vec<(String, Vec<u8>)>>>,
}

impl Intercept for SkipNonUtf8Headers {
    fn name(&self) -> &'static str {
        "SkipNonUtf8Headers"
    }

    fn read_before_execution(
        &self,
        _context: &BeforeSerializationInterceptorContextRef<'_>,
        cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        cfg.interceptor_state()
            .store_put(NonUtf8HeaderHandling::Skip);
        Ok(())
    }

    fn read_before_deserialization(
        &self,
        context: &BeforeDeserializationInterceptorContextRef<'_>,
        _runtime_components: &RuntimeComponents,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        // Runs once per attempt, so overwrite rather than append.
        *self.seen.lock().unwrap() = context
            .response()
            .headers()
            .iter_bytes()
            .filter(|(_, value)| std::str::from_utf8(value).is_err())
            .map(|(name, value)| (name.to_owned(), value.to_vec()))
            .collect();
        Ok(())
    }
}
```
"#
)]
///
/// This applies only to values bound to a modeled member. A header bound to nothing is never an
/// error regardless of encoding.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum NonUtf8HeaderHandling {
    /// Fail the operation, reporting the member and header that could not be parsed.
    ///
    /// This is the default: a value the service sent is not silently discarded.
    #[default]
    Reject,

    /// Deserialize the member as if the header were absent.
    ///
    /// The header itself is left in place, so the octets stay readable through
    /// [`Headers::get_bytes`](crate::http::Headers::get_bytes).
    ///
    /// Note this drops the whole member, not just the offending value: for a member bound to a
    /// list-valued header, one unreadable value makes the entire member `None`.
    Skip,
}

impl Storable for NonUtf8HeaderHandling {
    type Storer = StoreReplace<Self>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_smithy_types::config_bag::{CloneableLayer, ConfigBag};

    #[test]
    fn rejects_by_default() {
        assert_eq!(NonUtf8HeaderHandling::Reject, Default::default());
        // An empty bag must read as `Reject` rather than requiring callers to unwrap_or_default.
        let bag = ConfigBag::base();
        assert_eq!(None, bag.load::<NonUtf8HeaderHandling>());
    }

    #[test]
    fn round_trips_through_the_config_bag() {
        let mut layer = CloneableLayer::new("test");
        layer.store_put(NonUtf8HeaderHandling::Skip);
        let bag = ConfigBag::of_layers(vec![layer.into()]);
        assert_eq!(
            Some(&NonUtf8HeaderHandling::Skip),
            bag.load::<NonUtf8HeaderHandling>()
        );
    }
}
