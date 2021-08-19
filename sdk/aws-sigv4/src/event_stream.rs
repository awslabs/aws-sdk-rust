/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Utilities to sign Event Stream messages.

use crate::date_fmt::{format_date, format_date_time};
use crate::sign::{calculate_signature, generate_signing_key, sha256_hex_string};
use crate::SigningOutput;
use chrono::{DateTime, SubsecRound, Utc};
use smithy_eventstream::frame::{write_headers_to, Header, HeaderValue, Message};
use std::io::Write;

pub type SigningParams<'a> = super::SigningParams<'a, ()>;

/// Creates a string to sign for an Event Stream message.
fn calculate_string_to_sign(
    message_payload: &[u8],
    last_signature: &str,
    date_time: &DateTime<Utc>,
    params: &SigningParams<'_>,
) -> Vec<u8> {
    // Event Stream string to sign format is documented here:
    // https://docs.aws.amazon.com/transcribe/latest/dg/how-streaming.html
    let date_time_str = format_date_time(&date_time);
    let date_str = format_date(&date_time.date());

    let mut sts: Vec<u8> = Vec::new();
    writeln!(sts, "AWS4-HMAC-SHA256-PAYLOAD").unwrap();
    writeln!(sts, "{}", date_time_str).unwrap();
    writeln!(
        sts,
        "{}/{}/{}/aws4_request",
        date_str, params.region, params.service_name
    )
    .unwrap();
    writeln!(sts, "{}", last_signature).unwrap();

    let date_header = Header::new(":date", HeaderValue::Timestamp((*date_time).into()));
    let mut date_buffer = Vec::new();
    write_headers_to(&[date_header], &mut date_buffer).unwrap();
    writeln!(sts, "{}", sha256_hex_string(&date_buffer)).unwrap();
    write!(sts, "{}", sha256_hex_string(&message_payload)).unwrap();
    sts
}

/// Signs an Event Stream message with the given `credentials`.
///
/// Each message's signature incorporates the signature of the previous message (`last_signature`).
/// The very first message incorporates the signature of the top-level request
/// for both HTTP 2 and WebSocket.
pub fn sign_message<'a>(
    message: &'a Message,
    last_signature: &'a str,
    params: &'a SigningParams<'a>,
) -> SigningOutput<Message> {
    // Truncate the sub-seconds up front since the timestamp written to the signed message header
    // needs to exactly match the string formatted timestamp, which doesn't include sub-seconds.
    let date_time = params.date_time.trunc_subsecs(0);

    let signing_key = generate_signing_key(
        params.secret_key,
        date_time.date(),
        params.region,
        params.service_name,
    );
    let message_payload = {
        let mut payload = Vec::new();
        message.write_to(&mut payload).unwrap();
        payload
    };
    let string_to_sign =
        calculate_string_to_sign(&message_payload, last_signature, &date_time, params);
    let signature = calculate_signature(signing_key, &string_to_sign);

    // Generate the signed wrapper event frame
    SigningOutput::new(
        Message::new(message_payload)
            .add_header(Header::new(
                ":chunk-signature",
                HeaderValue::ByteArray(hex::decode(&signature).unwrap().into()),
            ))
            .add_header(Header::new(
                ":date",
                HeaderValue::Timestamp(date_time.into()),
            )),
        signature,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn string_to_sign() {
        let message_to_sign = Message::new(&b"test payload"[..]).add_header(Header::new(
            "some-header",
            HeaderValue::String("value".into()),
        ));
        let mut message_payload = Vec::new();
        message_to_sign.write_to(&mut message_payload).unwrap();

        let params = SigningParams {
            access_key: "fake access key",
            secret_key: "fake secret key",
            security_token: None,
            region: "us-east-1",
            service_name: "testservice",
            date_time: (UNIX_EPOCH + Duration::new(123_456_789_u64, 1234u32)).into(),
            settings: (),
        };

        let expected = "\
            AWS4-HMAC-SHA256-PAYLOAD\n\
            19731129T213309Z\n\
            19731129/us-east-1/testservice/aws4_request\n\
            be1f8c7d79ef8e1abc5254a2c70e4da3bfaf4f07328f527444e1fc6ea67273e2\n\
            0c0e3b3bf66b59b976181bd7d401927bbd624107303c713fd1e5f3d3c8dd1b1e\n\
            f2eba0f2e95967ee9fbc6db5e678d2fd599229c0d04b11e4fc8e0f2a02a806c6\
        ";

        let last_signature = sha256_hex_string(b"last message sts");
        assert_eq!(
            expected,
            std::str::from_utf8(&calculate_string_to_sign(
                &message_payload,
                &last_signature,
                &params.date_time,
                &params
            ))
            .unwrap()
        );
    }

    #[test]
    fn sign() {
        let message_to_sign = Message::new(&b"test payload"[..]).add_header(Header::new(
            "some-header",
            HeaderValue::String("value".into()),
        ));
        let params = SigningParams {
            access_key: "fake access key",
            secret_key: "fake secret key",
            security_token: None,
            region: "us-east-1",
            service_name: "testservice",
            date_time: (UNIX_EPOCH + Duration::new(123_456_789_u64, 1234u32)).into(),
            settings: (),
        };

        let last_signature = sha256_hex_string(b"last message sts");
        let (signed, signature) =
            sign_message(&message_to_sign, &last_signature, &params).into_parts();
        assert_eq!(":chunk-signature", signed.headers()[0].name().as_str());
        if let HeaderValue::ByteArray(bytes) = signed.headers()[0].value() {
            assert_eq!(signature, hex::encode(bytes));
        } else {
            panic!("expected byte array for :chunk-signature header");
        }
        assert_eq!(":date", signed.headers()[1].name().as_str());
        if let HeaderValue::Timestamp(value) = signed.headers()[1].value() {
            assert_eq!(123_456_789_i64, value.epoch_seconds());
            // The subseconds should have been truncated off
            assert_eq!(0, value.epoch_subsecond_nanos());
        } else {
            panic!("expected timestamp for :date header");
        }
    }
}
