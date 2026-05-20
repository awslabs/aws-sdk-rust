/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use criterion::{criterion_group, criterion_main, Criterion};

// TODO(schema-serde): Re-enable this benchmark when schema-serde codegen is
// active for DynamoDB (awsJson1_0). The body below exercises the schema-serde
// request serialization path, which requires a `SharedClientProtocol` in the
// config bag. With `SchemaSerdeAllowlist` empty on main, DynamoDB falls back
// to the legacy codegen path that does not consult the protocol. Once
// awsJson1_0 (or DynamoDB specifically) is re-added to the allowlist, replace
// the no-op `bench_group` below with the commented-out implementation.
// See: codegen-client/.../customizations/SchemaDecorator.kt
//
// --- BEGIN schema-serde bench (disabled) ---
/*
use aws_sdk_dynamodb::operation::put_item::{PutItem, PutItemInput};
use aws_sdk_dynamodb::types::AttributeValue;
use aws_smithy_runtime_api::client::interceptors::context::Input;
use aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugin;
use aws_smithy_runtime_api::client::ser_de::{SerializeRequest, SharedRequestSerializer};
use aws_smithy_types::config_bag::ConfigBag;

macro_rules! attr_s {
    ($str_val:expr) => {
        AttributeValue::S($str_val.into())
    };
}
macro_rules! attr_n {
    ($str_val:expr) => {
        AttributeValue::N($str_val.into())
    };
}
macro_rules! attr_list {
    ( $($attr_val:expr),* ) => {
        AttributeValue::L(vec![$($attr_val),*])
    }
}
macro_rules! attr_obj {
    { $($str_val:expr => $attr_val:expr),* } => {
        AttributeValue::M(
            vec![
                $(($str_val.to_string(), $attr_val)),*
            ].into_iter().collect()
        )
    };
}

fn do_bench(input: &PutItemInput) {
    let operation = PutItem::new();
    let config = operation.config().expect("operation should have config");
    let serializer = config
        .load::<SharedRequestSerializer>()
        .expect("operation should set a serializer");

    // Create a config bag with the required SharedClientProtocol
    let mut config_bag = ConfigBag::base();
    let protocol = aws_smithy_json::protocol::aws_json_rpc::AwsJsonRpcProtocol::aws_json_1_0(
        "DynamoDB_20120810",
    );
    let shared_protocol = aws_smithy_schema::protocol::SharedClientProtocol::new(protocol);
    let mut layer = aws_smithy_types::config_bag::Layer::new("bench");
    layer.store_put(shared_protocol);
    config_bag.push_shared_layer(layer.freeze());

    let input = Input::erase(input.clone());

    let request = serializer
        .serialize_input(input, &mut config_bag)
        .expect("success");
    let body = request.body().bytes().unwrap();
    assert_eq!(body[0], b'{');
}

fn bench_group(c: &mut Criterion) {
    c.bench_function("serialization_bench", |b| {
        let input = PutItemInput::builder()
            .table_name("Movies-5")
            .set_item(Some(
                attr_obj! {
                "year" => attr_n!("2013"),
                "title" => attr_s!("Turn It Down, Or Else!"),
                "info" => attr_obj! {
                    "directors" => attr_list![attr_s!("Alice Smith"), attr_s!("Bob Jones")],
                    "release_date" => attr_s!("2013-01-18T00:00:00Z"),
                    "rating" => attr_n!("6.2"),
                    "genres" => attr_list!(attr_s!("Comedy"), attr_s!("Drama")),
                    "image_url" => attr_s!("http://ia.media-imdb.com/images/N/O9ERWAU7FS797AJ7LU8HN09AMUP908RLlo5JF90EWR7LJKQ7@@._V1_SX400_.jpg"),
                    "plot" => attr_s!("A rock band plays their music at high volumes, annoying the neighbors."),
                    "rank" => attr_n!("11"),
                    "running_time_secs" => attr_n!("5215"),
                    "actors" => attr_list!(attr_s!("David Matthewman"), attr_s!("Ann Thomas"), attr_s!("Jonathan G. Neff"))
                }
            }.as_m().unwrap().clone(),
            ))
            .build()
            .expect("valid input");
        b.iter(|| do_bench(&input))
    });
}
*/
// --- END schema-serde bench (disabled) ---

fn bench_group(_c: &mut Criterion) {
    // no-op while schema-serde is disabled; see module note above.
}

criterion_group!(benches, bench_group);
criterion_main!(benches);
