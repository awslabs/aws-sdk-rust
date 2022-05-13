/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_json::deserialize::{Error, Token};
use aws_smithy_types::Number;
use serde_json::{Map, Value};
use std::iter::Peekable;

pub fn run_data(data: &[u8]) {
    // Parse through with aws-smithy-json first to make sure it doesn't panic on invalid inputs
    if let Ok(tokens) =
        aws_smithy_json::deserialize::json_token_iter(data).collect::<Result<Vec<Token>, Error>>()
    {
        // Exercise string unescaping since the later comparison against Serde
        // re-serializes, and thus, loses UTF-16 surrogate pairs.
        for token in tokens {
            if let Token::ValueString { value, .. } = token {
                if let Ok(unescaped) = value.to_unescaped() {
                    let serde_equiv =
                        serde_json::from_str::<String>(&format!("\"{}\"", value.as_escaped_str()))
                            .unwrap();
                    assert_eq!(serde_equiv, unescaped);
                }
            }
        }
    }

    // Now parse with Serde, and if it's valid, compare the two and panic if different
    let serde_value = serde_json::from_slice::<Value>(data);
    if let Ok(value) = serde_value {
        // Re-serialize to normalize the large floating point numbers
        let json = serde_json::to_string(&value).unwrap();

        let tokens = aws_smithy_json::deserialize::json_token_iter(json.as_bytes())
            .collect::<Result<Vec<Token>, Error>>()
            .unwrap();
        let mut token_iter = tokens.into_iter().peekable();
        let converted_value = convert_tokens(&mut token_iter);
        assert_eq!(None, token_iter.next());
        assert_eq!(value, converted_value);
    }
}

/// Converts a token stream into a Serde [Value]
fn convert_tokens<'a, I: Iterator<Item = Token<'a>>>(tokens: &mut Peekable<I>) -> Value {
    match tokens.next().unwrap() {
        Token::StartObject { .. } => {
            let mut map = Map::new();
            loop {
                match tokens.next() {
                    Some(Token::EndObject { .. }) => break,
                    Some(Token::ObjectKey { key, .. }) => {
                        let key = key.to_unescaped().unwrap().to_string();
                        let value = convert_tokens(tokens);
                        map.insert(key, value);
                    }
                    Some(_) => unreachable!(),
                    None => panic!("should have encountered EndObject before end of stream"),
                }
            }
            Value::Object(map)
        }
        Token::StartArray { .. } => {
            let mut list = Vec::new();
            loop {
                match tokens.peek() {
                    Some(Token::EndArray { .. }) => {
                        tokens.next();
                        break;
                    }
                    Some(_) => {
                        list.push(convert_tokens(tokens));
                    }
                    None => panic!("should have encountered EndArray before end of stream"),
                }
            }
            Value::Array(list)
        }
        Token::ValueNull { .. } => Value::Null,
        Token::ValueNumber { value, .. } => Value::Number(match value {
            Number::NegInt(value) => serde_json::Number::from(value),
            Number::PosInt(value) => serde_json::Number::from(value),
            Number::Float(value) => serde_json::Number::from_f64(value).unwrap(),
        }),
        Token::ValueString { value, .. } => Value::String(value.to_unescaped().unwrap().into()),
        Token::ValueBool { value, .. } => Value::Bool(value),
        _ => unreachable!(),
    }
}
