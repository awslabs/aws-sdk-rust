/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_config::meta::region::RegionProviderChain;
use aws_config::{BehaviorVersion, Region};
use std::collections::HashMap;
use std::fmt;
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id};
use tracing::Subscriber;
use tracing_fluent_assertions::{AssertionRegistry, AssertionsLayer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::layer::{Context, Layer};
use tracing_subscriber::registry::LookupSpan;
mod utils;
use utils::{make_config, make_ddb_call, make_s3_call};

#[tokio::test]
async fn top_level_spans_exist_with_correct_attributes() {
    let s3_top_level: fn() -> Box<dyn Visit + 'static> = || Box::new(S3TestVisitor);
    let ddb_top_level: fn() -> Box<dyn Visit + 'static> = || Box::new(DdbTestVisitor);
    let subscriber = tracing_subscriber::registry::Registry::default().with(TestLayer {
        visitor_factories: HashMap::from([
            ("s3.GetObject", s3_top_level),
            ("dynamodb.GetItem", ddb_top_level),
        ]),
    });
    let _guard = tracing::subscriber::set_default(subscriber);

    let config = make_config(false);
    make_s3_call(&config).await;
    make_ddb_call(&config).await;
}

#[tokio::test]
async fn try_attempt_spans_emitted_per_retry() {
    let assertion_registry = AssertionRegistry::default();
    let base_subscriber = tracing_subscriber::Registry::default();
    let subscriber = base_subscriber.with(AssertionsLayer::new(&assertion_registry));
    let _guard = tracing::subscriber::set_default(subscriber);

    let two_try_attempts = assertion_registry
        .build()
        .with_name("try_attempt")
        .with_span_field("attempt")
        .was_closed_exactly(2)
        .finalize();

    let config = make_config(true);
    make_s3_call(&config).await;

    two_try_attempts.assert();
}

#[tokio::test]
async fn all_expected_operation_spans_emitted_with_correct_nesting() {
    let assertion_registry = AssertionRegistry::default();
    let base_subscriber = tracing_subscriber::Registry::default();
    let subscriber = base_subscriber.with(AssertionsLayer::new(&assertion_registry));
    let _guard = tracing::subscriber::set_default(subscriber);

    const OPERATION_NAME: &str = "s3.GetObject";
    const INVOKE: &str = "invoke";
    const TRY_OP: &str = "try_op";
    const TRY_ATTEMPT: &str = "try_attempt";

    let apply_configuration = assertion_registry
        .build()
        .with_name("apply_configuration")
        .with_parent_name(OPERATION_NAME)
        .with_parent_name(INVOKE)
        .was_closed_exactly(1)
        .finalize();

    let serialization = assertion_registry
        .build()
        .with_name("serialization")
        .with_parent_name(OPERATION_NAME)
        .with_parent_name(INVOKE)
        .with_parent_name(TRY_OP)
        .was_closed_exactly(1)
        .finalize();

    let orchestrate_endpoint = assertion_registry
        .build()
        .with_name("orchestrate_endpoint")
        .with_parent_name(OPERATION_NAME)
        .with_parent_name(INVOKE)
        .with_parent_name(TRY_OP)
        .with_parent_name(TRY_ATTEMPT)
        .was_closed_exactly(1)
        .finalize();

    let lazy_load_identity = assertion_registry
        .build()
        .with_name("lazy_load_identity")
        .with_parent_name(OPERATION_NAME)
        .with_parent_name(INVOKE)
        .with_parent_name(TRY_OP)
        .with_parent_name(TRY_ATTEMPT)
        .was_closed_exactly(1)
        .finalize();

    let deserialize_streaming = assertion_registry
        .build()
        .with_name("deserialize_streaming")
        .with_parent_name(OPERATION_NAME)
        .with_parent_name(INVOKE)
        .with_parent_name(TRY_OP)
        .with_parent_name(TRY_ATTEMPT)
        .was_closed_exactly(1)
        .finalize();

    let deserialization = assertion_registry
        .build()
        .with_name("deserialization")
        .with_parent_name(OPERATION_NAME)
        .with_parent_name(INVOKE)
        .with_parent_name(TRY_OP)
        .with_parent_name(TRY_ATTEMPT)
        .was_closed_exactly(1)
        .finalize();

    let try_attempt = assertion_registry
        .build()
        .with_name(TRY_ATTEMPT)
        .with_span_field("attempt")
        .with_parent_name(OPERATION_NAME)
        .with_parent_name(INVOKE)
        .with_parent_name(TRY_OP)
        .was_closed_exactly(1)
        .finalize();

    let finally_attempt = assertion_registry
        .build()
        .with_name("finally_attempt")
        .with_parent_name(OPERATION_NAME)
        .with_parent_name(INVOKE)
        .with_parent_name(TRY_OP)
        .was_closed_exactly(1)
        .finalize();

    let try_op = assertion_registry
        .build()
        .with_name(TRY_OP)
        .with_parent_name(OPERATION_NAME)
        .with_parent_name(INVOKE)
        .was_closed_exactly(1)
        .finalize();

    let finally_op = assertion_registry
        .build()
        .with_name("finally_op")
        .with_parent_name(OPERATION_NAME)
        .with_parent_name(INVOKE)
        .was_closed_exactly(1)
        .finalize();

    let invoke = assertion_registry
        .build()
        .with_name(INVOKE)
        .with_parent_name(OPERATION_NAME)
        .was_closed_exactly(1)
        .finalize();

    let operation = assertion_registry
        .build()
        .with_name(OPERATION_NAME)
        .was_closed_exactly(1)
        .finalize();

    let config = make_config(false);
    make_s3_call(&config).await;

    apply_configuration.assert();
    serialization.assert();
    orchestrate_endpoint.assert();
    lazy_load_identity.assert();
    deserialize_streaming.assert();
    deserialization.assert();
    try_attempt.assert();
    finally_attempt.assert();
    try_op.assert();
    finally_op.assert();
    invoke.assert();
    operation.assert();
}

#[tokio::test]
async fn config_spans_emitted() {
    let assertion_registry = AssertionRegistry::default();
    let base_subscriber = tracing_subscriber::Registry::default();
    let subscriber = base_subscriber.with(AssertionsLayer::new(&assertion_registry));
    let _guard = tracing::subscriber::set_default(subscriber);

    let load_config_file = assertion_registry
        .build()
        .with_name("load_config_file")
        .with_span_field("file")
        .was_closed_exactly(2)
        .finalize();

    let build_profile_file_credentials_provider = assertion_registry
        .build()
        .with_name("build_profile_file_credentials_provider")
        .was_closed_exactly(1)
        .finalize();

    let build_profile_token_provider = assertion_registry
        .build()
        .with_name("build_profile_token_provider")
        .was_closed_exactly(1)
        .finalize();

    let _config = aws_config::defaults(BehaviorVersion::latest())
        .region(Region::from_static("foo"))
        .load()
        .await;

    load_config_file.assert();
    build_profile_file_credentials_provider.assert();
    build_profile_token_provider.assert();
}

// NOTE: this test is being temporarily ignored since, although it succeeds both locally and in the
// GitHub CI, it fails in our CodeBuild CI, likely because CodeBuild runs on EC2 so IMDS is present
// and causes different behavior.
#[tokio::test]
#[ignore]
async fn region_spans_emitted() {
    let assertion_registry = AssertionRegistry::default();
    let base_subscriber = tracing_subscriber::Registry::default();
    let subscriber = base_subscriber.with(AssertionsLayer::new(&assertion_registry));
    let _guard = tracing::subscriber::set_default(subscriber);

    let region_provider_chain = assertion_registry
        .build()
        .with_name("region_provider_chain")
        .with_span_field("provider")
        .was_closed_exactly(5)
        .finalize();

    let imds_load_region = assertion_registry
        .build()
        .with_name("imds_load_region")
        .with_parent_name("region_provider_chain")
        .was_closed_exactly(1)
        .finalize();

    // IMDS calls invoke twice, once with get and once with get_token
    let invoke = assertion_registry
        .build()
        .with_name("invoke")
        .with_parent_name("imds_load_region")
        .was_closed_exactly(2)
        .finalize();

    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let _config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;

    region_provider_chain.assert();
    imds_load_region.assert();
    invoke.assert();
}

/// Layer for testing top-level spans. Takes in a hashmap where the keys are the names of top level spans
/// and the values are structs implementing [Visit] that make assertions about the attributes of those spans
struct TestLayer<F: Fn() -> Box<dyn Visit> + 'static> {
    visitor_factories: HashMap<&'static str, F>,
}

impl<S, F> Layer<S> for TestLayer<F>
where
    S: Subscriber,
    S: for<'lookup> LookupSpan<'lookup>,
    F: Fn() -> Box<dyn Visit>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let span = ctx.span(id).unwrap();
        let span_name = span.metadata().name();

        // Assert that any top level spans are the operation spans from
        // visitor_factories
        if span.parent().is_none() {
            assert!(
                self.visitor_factories.contains_key(span_name),
                "Encountered unexpected top level span {span_name}"
            )
        }

        for (asserted_span, visitor_factory) in &self.visitor_factories {
            if &span_name == asserted_span {
                let mut visitor = visitor_factory();
                attrs.values().record(&mut *visitor);
            }
        }
    }
}

struct S3TestVisitor;

impl Visit for S3TestVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        let field_name = field.name();
        let field_value = format!("{value:?}").replace("\"", "");
        if field_name == "rpc.system" {
            assert_eq!("aws-api".to_string(), field_value);
        } else if field_name == "rpc.service" {
            assert_eq!("s3".to_string(), field_value);
        } else if field_name == "rpc.method" {
            assert_eq!("GetObject".to_string(), field_value);
        } else if field_name == "sdk_invocation_id" {
            let num: u32 = field_value.parse().unwrap();
            assert!(1_000_000 <= num);
            assert!(num < 10_000_000);
        } else {
            panic!("Unknown attribute present on top level operation span - {field_name}: {field_value}")
        }
    }
}

struct DdbTestVisitor;

impl Visit for DdbTestVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        let field_name = field.name();
        let field_value = format!("{value:?}").replace("\"", "");
        if field_name == "rpc.system" {
            assert_eq!("aws-api".to_string(), field_value);
        } else if field_name == "rpc.service" {
            assert_eq!("dynamodb".to_string(), field_value);
        } else if field_name == "rpc.method" {
            assert_eq!("GetItem".to_string(), field_value);
        } else if field_name == "sdk_invocation_id" {
            let num: u32 = field_value.parse().unwrap();
            assert!(1_000_000 <= num);
            assert!(num < 10_000_000);
        } else {
            panic!("Unknown attribute present on top level operation span - {field_name}: {field_value}")
        }
    }
}
