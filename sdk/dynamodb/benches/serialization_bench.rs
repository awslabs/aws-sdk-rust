/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_dynamodb::operation::put_item::PutItemInput;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Config;
use criterion::{criterion_group, criterion_main, Criterion};
use futures_util::FutureExt;

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

fn do_bench(config: &Config, input: &PutItemInput) {
    let operation = input
        .make_operation(&config)
        .now_or_never()
        .unwrap()
        .expect("operation failed to build");
    let (http_request, _parts) = operation.into_request_response().0.into_parts();
    let body = http_request.body().bytes().unwrap();
    assert_eq!(body[0], b'{');
}

fn bench_group(c: &mut Criterion) {
    c.bench_function("serialization_bench", |b| {
        let config = Config::builder().build();
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
        b.iter(|| do_bench(&config, &input))
    });
}

criterion_group!(benches, bench_group);
criterion_main!(benches);
