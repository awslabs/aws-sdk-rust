/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::{pretty_comparison, ProtocolTestFailure};
use regex::Regex;

fn rewrite_url_encoded_map_keys(input: &str) -> (String, String) {
    let mut itr = input.split('=');
    let (key, value) = (itr.next().unwrap(), itr.next().unwrap());

    let regex = Regex::new(r"^(.+)\.\d+\.(.+)$").unwrap();
    if let Some(captures) = regex.captures(key) {
        let rewritten_key = format!(
            "{}.N.{}",
            captures.get(1).unwrap().as_str(),
            captures.get(2).unwrap().as_str()
        );
        (rewritten_key, value.to_string())
    } else {
        (key.to_string(), value.to_string())
    }
}

fn rewrite_url_encoded_body(input: &str) -> String {
    let mut entries: Vec<(String, String)> = input
        .split('&')
        .map(|entry| entry.trim())
        .filter(|s| !s.is_empty())
        .map(rewrite_url_encoded_map_keys)
        .collect();
    if entries.len() > 2 {
        entries[2..].sort_by(|a, b| a.1.cmp(&b.1));
    }
    let entries: Vec<String> = entries
        .into_iter()
        .map(|kv| format!("{}={}", kv.0, kv.1))
        .collect();
    entries.join("\n&")
}

pub(crate) fn try_url_encoded_form_equivalent(
    actual: &str,
    expected: &str,
) -> Result<(), ProtocolTestFailure> {
    let actual = rewrite_url_encoded_body(actual);
    let expected = rewrite_url_encoded_body(expected);
    if actual == expected {
        Ok(())
    } else {
        Err(ProtocolTestFailure::BodyDidNotMatch {
            comparison: pretty_comparison(&actual, &expected),
            hint: "".into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::urlencoded::try_url_encoded_form_equivalent;

    #[test]
    fn test_url_encoded_form_equivalent() {
        assert_eq!(
            Ok(()),
            try_url_encoded_form_equivalent(
                "Action=Something&Version=test",
                "Action=Something&Version=test",
            )
        );

        assert!(try_url_encoded_form_equivalent(
            "Action=Something&Version=test&Property=foo",
            "Action=Something&Version=test&Property=bar",
        )
        .is_err());

        assert!(try_url_encoded_form_equivalent(
            "Action=Something&Version=test&WrongProperty=foo",
            "Action=Something&Version=test&Property=foo",
        )
        .is_err());

        assert_eq!(
            Ok(()),
            try_url_encoded_form_equivalent(
                "Action=Something&Version=test\
                &SomeMap.1.key=foo\
                &SomeMap.1.value=Foo\
                &SomeMap.2.key=bar\
                &SomeMap.2.value=Bar",
                "Action=Something&Version=test\
                &SomeMap.1.key=bar\
                &SomeMap.1.value=Bar\
                &SomeMap.2.key=foo\
                &SomeMap.2.value=Foo",
            )
        );
    }
}
