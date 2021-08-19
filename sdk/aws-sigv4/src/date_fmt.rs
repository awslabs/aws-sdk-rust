/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

// Some of the functions in this file are unused when disabling certain features
#![allow(dead_code)]
use chrono::{Date, DateTime, NaiveDate, NaiveDateTime, ParseError, Utc};

const DATE_TIME_FORMAT: &str = "%Y%m%dT%H%M%SZ";
const DATE_FORMAT: &str = "%Y%m%d";

/// Formats a chrono `Date<Utc>` in `YYYYMMDD` format.
pub fn format_date(date: &Date<Utc>) -> String {
    date.format(DATE_FORMAT).to_string()
}

/// Parses `YYYYMMDD` formatted dates into a chrono `Date<Utc>`.
pub fn parse_date(date_str: &str) -> Result<Date<Utc>, ParseError> {
    Ok(Date::<Utc>::from_utc(
        NaiveDate::parse_from_str(date_str, "%Y%m%d")?,
        Utc,
    ))
}

/// Formats a chrono `DateTime<Utc>` in `YYYYMMDD'T'HHMMSS'Z'` format.
pub fn format_date_time(date_time: &DateTime<Utc>) -> String {
    date_time.format(DATE_TIME_FORMAT).to_string()
}

/// Parses `YYYYMMDD'T'HHMMSS'Z'` formatted dates into a chrono `DateTime<Utc>`.
pub fn parse_date_time(date_time_str: &str) -> Result<DateTime<Utc>, ParseError> {
    Ok(DateTime::<Utc>::from_utc(
        NaiveDateTime::parse_from_str(date_time_str, DATE_TIME_FORMAT)?,
        Utc,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_time_roundtrip() {
        let date = parse_date_time("20150830T123600Z").unwrap();
        assert_eq!("20150830T123600Z", format_date_time(&date));
    }

    #[test]
    fn date_roundtrip() {
        let date = parse_date("20150830").unwrap();
        assert_eq!("20150830", format_date(&date));
    }
}
