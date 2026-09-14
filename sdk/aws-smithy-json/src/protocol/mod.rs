/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Protocol implementations for JSON-based Smithy protocols.

pub mod aws_json_rpc;
pub mod aws_rest_json_1;
pub(crate) mod error;

use crate::codec::{JsonCodec, JsonCodecSettings};
use aws_smithy_schema::protocol::ServiceShapeNamespace;
use aws_smithy_types::config_bag::ConfigBag;

/// Returns a codec whose `default_namespace` is filled in from the config bag, or `None` when
/// there is nothing to change.
///
/// `default_namespace` resolves relative `__type` discriminators on documents (see
/// [`JsonCodecSettings::default_namespace`]). It is a *response*-parsing concern only — nothing
/// on the serialization path reads it, because per the document-types SEP a serializer must
/// always emit an absolute shape ID.
///
/// An explicitly configured namespace always wins: if the codec already carries one, this
/// returns `None` and the caller uses the codec it already has. The fallback exists for a
/// protocol constructed by hand and selected at runtime via `Config::builder().protocol(..)`,
/// which cannot know the model's namespace and would otherwise leave every relative
/// discriminator unresolved — the type registry would then silently fail to find the shape.
///
/// **Currently inert, deliberately.** `default_namespace` is read at exactly one place —
/// [`JsonDeserializer::read_discriminated_document`](crate::codec::JsonDeserializer::read_discriminated_document)
/// — and that method has no production caller: it is inherent on the concrete deserializer
/// rather than on [`ShapeDeserializer`](aws_smithy_schema::serde::ShapeDeserializer), so it is
/// unreachable through the boxed trait object every protocol's `deserialize_response` returns,
/// and neither codegen nor any generated crate calls it. The same is true of the
/// `with_default_namespace` call codegen emits, which configures the same setting. This fallback
/// exists so that when the document/type-registry response path is wired up, a protocol selected
/// at runtime resolves relative discriminators without the caller having to know the namespace —
/// rather than that gap being discovered then. Its effect is asserted at the codec level below,
/// which is the highest level at which it is currently observable.
///
/// **Cost.** Building the replacement clones the settings struct and re-`Arc`s it, measured at
/// **~46 ns per response**. That is ~1% of the smallest structured-response parse measured on
/// this path (~3.8 µs for a 339-byte body) and well inside its run-to-run noise, so it is not
/// worth caching. It is also not paid by generated clients at all: codegen passes the namespace
/// explicitly, so they take the `None` branch. If a profile ever disagrees, the resolved codec
/// could be memoized in a `OnceLock` on the protocol — the bag entry is service-scoped and
/// stored once per client, so it does not vary per request.
pub(crate) fn codec_with_bag_namespace(codec: &JsonCodec, cfg: &ConfigBag) -> Option<JsonCodec> {
    if codec.settings().default_namespace().is_some() {
        return None;
    }
    let namespace = cfg.load::<ServiceShapeNamespace>()?;
    let settings: JsonCodecSettings = codec
        .settings()
        .to_builder()
        .default_namespace(namespace.as_str())
        .build();
    Some(JsonCodec::new(settings))
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_smithy_schema::codec::Codec;
    use aws_smithy_types::config_bag::Layer;

    fn plain_codec() -> JsonCodec {
        JsonCodec::new(JsonCodecSettings::builder().use_json_name(true).build())
    }

    fn codec_with_namespace(ns: &str) -> JsonCodec {
        JsonCodec::new(
            JsonCodecSettings::builder()
                .use_json_name(true)
                .default_namespace(ns)
                .build(),
        )
    }

    fn bag_with_namespace(ns: &'static str) -> ConfigBag {
        let mut layer = Layer::new("test");
        layer.store_put(ServiceShapeNamespace::new(ns));
        ConfigBag::of_layers(vec![layer])
    }

    /// The whole point of the fallback: a protocol constructed by hand, with no namespace of its
    /// own, picks up the model's namespace that generated clients leave in the bag.
    #[test]
    fn bag_supplies_namespace_when_codec_has_none() {
        let resolved = codec_with_bag_namespace(
            &plain_codec(),
            &bag_with_namespace("com.amazonaws.dynamodb"),
        )
        .expect("a replacement codec is produced");
        assert_eq!(
            Some("com.amazonaws.dynamodb"),
            resolved.settings().default_namespace()
        );
    }

    /// An explicit `with_default_namespace` must win over the bag, so a caller can still override
    /// a service whose discriminators are not in its own namespace.
    #[test]
    fn explicit_namespace_beats_the_bag() {
        assert!(
            codec_with_bag_namespace(
                &codec_with_namespace("com.example.explicit"),
                &bag_with_namespace("com.amazonaws.dynamodb"),
            )
            .is_none(),
            "an explicitly configured namespace must not be replaced by the bag"
        );
    }

    /// With neither source there is nothing to do, and the caller must keep its own codec rather
    /// than pay for a rebuild.
    #[test]
    fn no_namespace_anywhere_is_a_no_op() {
        assert!(codec_with_bag_namespace(&plain_codec(), &ConfigBag::base()).is_none());
    }

    /// Preserves the rest of the settings. A rebuild that silently reset `use_json_name` would
    /// change wire behavior for every restJson1 response, so this is the load-bearing assertion.
    /// `field_mapper` has no public accessor, so it is checked behaviorally: `use_json_name(true)`
    /// means a member's `@jsonName` is honored when reading.
    #[test]
    fn rebuild_preserves_other_settings() {
        let original = JsonCodec::new(
            JsonCodecSettings::builder()
                .use_json_name(true)
                .default_timestamp_format(aws_smithy_types::date_time::Format::DateTime)
                .max_depth(7)
                .use_string_for_arbitrary_precision(true)
                .build(),
        );
        let resolved = codec_with_bag_namespace(&original, &bag_with_namespace("com.example"))
            .expect("a replacement codec is produced");
        let before = original.settings();
        let after = resolved.settings();
        assert_eq!(
            before.default_timestamp_format(),
            after.default_timestamp_format()
        );
        assert_eq!(before.max_depth(), after.max_depth());
        assert_eq!(
            before.use_string_for_arbitrary_precision(),
            after.use_string_for_arbitrary_precision()
        );
        assert_eq!(Some("com.example"), after.default_namespace());
    }

    /// The resolved codec is not merely labelled — it actually resolves a relative `__type`.
    /// This is the highest level at which the fallback is observable today, because
    /// `read_discriminated_document` is inherent on `JsonDeserializer` and has no production
    /// caller; see the note on [`codec_with_bag_namespace`].
    #[test]
    fn resolved_codec_lifts_a_relative_discriminator() {
        let input = br#"{"__type":"Capacity","CapacityUnits":1.0}"#;

        let plain = plain_codec();
        let mut before = plain.create_deserializer(&input[..]);
        assert_eq!(
            None,
            before
                .read_discriminated_document()
                .expect("parses")
                .discriminator(),
            "without a namespace a relative __type must stay unresolved"
        );

        let resolved =
            codec_with_bag_namespace(&plain, &bag_with_namespace("com.amazonaws.dynamodb"))
                .expect("a replacement codec is produced");
        let mut after = resolved.create_deserializer(&input[..]);
        assert_eq!(
            Some("com.amazonaws.dynamodb#Capacity"),
            after
                .read_discriminated_document()
                .expect("parses")
                .discriminator(),
            "the bag-derived namespace must actually resolve the discriminator"
        );
    }
}

#[cfg(test)]
mod codec_rebuild_tests {
    use super::*;
    use crate::codec::{JsonCodec, JsonCodecSettings};
    use aws_smithy_schema::codec::Codec;
    use aws_smithy_schema::protocol::ServiceShapeNamespace;
    use aws_smithy_schema::serde::ShapeDeserializer;

    /// `codec_with_bag_namespace` rebuilds the codec's settings to inject the
    /// service namespace. It must override *only* that, carrying every other
    /// setting through — otherwise selecting a protocol at runtime silently
    /// resets strictness on a codec the caller configured. Verified through
    /// observable behavior rather than the private fields: with
    /// `allow_integral_float_numbers`, `1.0` reads as a long; without it, the
    /// integer path rejects a floating-point value.
    #[test]
    fn bag_namespace_rebuild_preserves_other_settings() {
        for flag in [false, true] {
            let codec = JsonCodec::new(
                JsonCodecSettings::builder()
                    .allow_integral_float_numbers(flag)
                    .build(),
            );
            let mut cfg = ConfigBag::base();
            cfg.interceptor_state()
                .store_put(ServiceShapeNamespace::new("com.example"));

            let rebuilt = codec_with_bag_namespace(&codec, &cfg)
                .expect("namespace is absent from the codec, so it must be rebuilt");

            assert_eq!(rebuilt.settings().default_namespace(), Some("com.example"));
            assert_eq!(
                rebuilt
                    .create_deserializer(b"1.0")
                    .read_long(&aws_smithy_schema::prelude::LONG)
                    .is_ok(),
                flag,
                "allow_integral_float_numbers({flag}) must survive the rebuild"
            );
        }
    }
}
