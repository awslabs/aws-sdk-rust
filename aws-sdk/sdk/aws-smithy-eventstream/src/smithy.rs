/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use crate::error::Error;
use crate::frame::{Header, HeaderValue, Message};
use crate::str_bytes::StrBytes;
use aws_smithy_types::{Blob, DateTime};

macro_rules! expect_shape_fn {
    (fn $fn_name:ident[$val_typ:ident] -> $result_typ:ident { $val_name:ident -> $val_expr:expr }) => {
        pub fn $fn_name(header: &Header) -> Result<$result_typ, Error> {
            match header.value() {
                HeaderValue::$val_typ($val_name) => Ok($val_expr),
                _ => Err(Error::Unmarshalling(format!(
                    "expected '{}' header value to be {}",
                    header.name().as_str(),
                    stringify!($val_typ)
                ))),
            }
        }
    };
}

expect_shape_fn!(fn expect_bool[Bool] -> bool { value -> *value });
expect_shape_fn!(fn expect_byte[Byte] -> i8 { value -> *value });
expect_shape_fn!(fn expect_int16[Int16] -> i16 { value -> *value });
expect_shape_fn!(fn expect_int32[Int32] -> i32 { value -> *value });
expect_shape_fn!(fn expect_int64[Int64] -> i64 { value -> *value });
expect_shape_fn!(fn expect_byte_array[ByteArray] -> Blob { bytes -> Blob::new(bytes.as_ref()) });
expect_shape_fn!(fn expect_string[String] -> String { value -> value.as_str().into() });
expect_shape_fn!(fn expect_timestamp[Timestamp] -> DateTime { value -> *value });

#[derive(Debug)]
pub struct ResponseHeaders<'a> {
    pub content_type: Option<&'a StrBytes>,
    pub message_type: &'a StrBytes,
    pub smithy_type: &'a StrBytes,
}

impl<'a> ResponseHeaders<'a> {
    pub fn content_type(&self) -> Option<&str> {
        self.content_type.map(|ct| ct.as_str())
    }
}

fn expect_header_str_value<'a>(
    header: Option<&'a Header>,
    name: &str,
) -> Result<&'a StrBytes, Error> {
    match header {
        Some(header) => Ok(header.value().as_string().map_err(|value| {
            Error::Unmarshalling(format!(
                "expected response {} header to be string, received {:?}",
                name, value
            ))
        })?),
        None => Err(Error::Unmarshalling(format!(
            "expected response to include {} header, but it was missing",
            name
        ))),
    }
}

pub fn parse_response_headers(message: &Message) -> Result<ResponseHeaders, Error> {
    let (mut content_type, mut message_type, mut event_type, mut exception_type) =
        (None, None, None, None);
    for header in message.headers() {
        match header.name().as_str() {
            ":content-type" => content_type = Some(header),
            ":message-type" => message_type = Some(header),
            ":event-type" => event_type = Some(header),
            ":exception-type" => exception_type = Some(header),
            _ => {}
        }
    }
    let message_type = expect_header_str_value(message_type, ":message-type")?;
    Ok(ResponseHeaders {
        content_type: content_type
            .map(|ct| expect_header_str_value(Some(ct), ":content-type"))
            .transpose()?,
        message_type,
        smithy_type: if message_type.as_str() == "event" {
            expect_header_str_value(event_type, ":event-type")?
        } else if message_type.as_str() == "exception" {
            expect_header_str_value(exception_type, ":exception-type")?
        } else {
            return Err(Error::Unmarshalling(format!(
                "unrecognized `:message-type`: {}",
                message_type.as_str()
            )));
        },
    })
}

#[cfg(test)]
mod tests {
    use super::parse_response_headers;
    use crate::frame::{Header, HeaderValue, Message};

    #[test]
    fn normal_message() {
        let message = Message::new(&b"test"[..])
            .add_header(Header::new(
                ":event-type",
                HeaderValue::String("Foo".into()),
            ))
            .add_header(Header::new(
                ":content-type",
                HeaderValue::String("application/json".into()),
            ))
            .add_header(Header::new(
                ":message-type",
                HeaderValue::String("event".into()),
            ));
        let parsed = parse_response_headers(&message).unwrap();
        assert_eq!("Foo", parsed.smithy_type.as_str());
        assert_eq!(Some("application/json"), parsed.content_type());
        assert_eq!("event", parsed.message_type.as_str());
    }

    #[test]
    fn error_message() {
        let message = Message::new(&b"test"[..])
            .add_header(Header::new(
                ":exception-type",
                HeaderValue::String("BadRequestException".into()),
            ))
            .add_header(Header::new(
                ":content-type",
                HeaderValue::String("application/json".into()),
            ))
            .add_header(Header::new(
                ":message-type",
                HeaderValue::String("exception".into()),
            ));
        let parsed = parse_response_headers(&message).unwrap();
        assert_eq!("BadRequestException", parsed.smithy_type.as_str());
        assert_eq!(Some("application/json"), parsed.content_type());
        assert_eq!("exception", parsed.message_type.as_str());
    }

    #[test]
    fn missing_exception_type() {
        let message = Message::new(&b"test"[..])
            .add_header(Header::new(
                ":content-type",
                HeaderValue::String("application/json".into()),
            ))
            .add_header(Header::new(
                ":message-type",
                HeaderValue::String("exception".into()),
            ));
        let error = parse_response_headers(&message).err().unwrap().to_string();
        assert_eq!(
            "failed to unmarshall message: expected response to include :exception-type \
             header, but it was missing",
            error
        );
    }

    #[test]
    fn missing_event_type() {
        let message = Message::new(&b"test"[..])
            .add_header(Header::new(
                ":content-type",
                HeaderValue::String("application/json".into()),
            ))
            .add_header(Header::new(
                ":message-type",
                HeaderValue::String("event".into()),
            ));
        let error = parse_response_headers(&message).err().unwrap().to_string();
        assert_eq!(
            "failed to unmarshall message: expected response to include :event-type \
             header, but it was missing",
            error
        );
    }

    #[test]
    fn missing_content_type() {
        let message = Message::new(&b"test"[..])
            .add_header(Header::new(
                ":event-type",
                HeaderValue::String("Foo".into()),
            ))
            .add_header(Header::new(
                ":message-type",
                HeaderValue::String("event".into()),
            ));
        let parsed = parse_response_headers(&message).ok().unwrap();
        assert_eq!(None, parsed.content_type);
        assert_eq!("Foo", parsed.smithy_type.as_str());
        assert_eq!("event", parsed.message_type.as_str());
    }

    #[test]
    fn content_type_wrong_type() {
        let message = Message::new(&b"test"[..])
            .add_header(Header::new(
                ":event-type",
                HeaderValue::String("Foo".into()),
            ))
            .add_header(Header::new(":content-type", HeaderValue::Int32(16)))
            .add_header(Header::new(
                ":message-type",
                HeaderValue::String("event".into()),
            ));
        let error = parse_response_headers(&message).err().unwrap().to_string();
        assert_eq!(
            "failed to unmarshall message: expected response :content-type \
             header to be string, received Int32(16)",
            error
        );
    }
}
