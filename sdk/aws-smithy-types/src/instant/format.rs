/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use std::error::Error;
use std::fmt;

const NANOS_PER_SECOND: u32 = 1_000_000_000;

#[non_exhaustive]
#[derive(Debug, Eq, PartialEq)]
pub enum DateParseError {
    Invalid(&'static str),
    IntParseError,
}

impl Error for DateParseError {}

impl fmt::Display for DateParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use DateParseError::*;
        match self {
            Invalid(msg) => write!(f, "invalid date: {}", msg),
            IntParseError => write!(f, "failed to parse int"),
        }
    }
}

pub(crate) mod http_date {
    use std::str::FromStr;

    use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Weekday};

    use crate::Instant;
    // This code is taken from https://github.com/pyfisch/httpdate and modified under an
    // Apache 2.0 License. Modifications:
    // - Removed use of unsafe
    // - Add serialization and deserialization of subsecond nanos
    use crate::instant::format::{DateParseError, NANOS_PER_SECOND};

    /// Format an `instant` in the HTTP date format (imf-fixdate) with added support for subsecond precision
    ///
    /// Example: "Mon, 16 Dec 2019 23:48:18 GMT"
    ///
    /// Some notes:
    /// - HTTP date does not support years before `0000`—this will cause a panic.
    /// - If you _don't_ want subsecond precision (e.g. if you want strict adherence to the spec),
    ///   you need to zero-out the instant before formatting
    /// - If subsecond nanos are 0, no fractional seconds are added
    /// - If subsecond nanos are nonzero, 3 digits of fractional seconds are added
    pub(crate) fn format(instant: &Instant) -> String {
        let structured = instant.to_chrono_internal();
        let weekday = match structured.weekday() {
            Weekday::Mon => "Mon",
            Weekday::Tue => "Tue",
            Weekday::Wed => "Wed",
            Weekday::Thu => "Thu",
            Weekday::Fri => "Fri",
            Weekday::Sat => "Sat",
            Weekday::Sun => "Sun",
        };
        let month = match structured.month() {
            1 => "Jan",
            2 => "Feb",
            3 => "Mar",
            4 => "Apr",
            5 => "May",
            6 => "Jun",
            7 => "Jul",
            8 => "Aug",
            9 => "Sep",
            10 => "Oct",
            11 => "Nov",
            12 => "Dec",
            _ => unreachable!(),
        };
        let mut out = String::with_capacity(32);
        fn push_digit(out: &mut String, digit: u8) {
            out.push((b'0' + digit as u8) as char);
        }

        out.push_str(weekday);
        out.push_str(", ");
        let day = structured.date().day() as u8;
        push_digit(&mut out, day / 10);
        push_digit(&mut out, day % 10);

        out.push(' ');
        out.push_str(month);

        out.push(' ');

        let year = structured.year();
        // Although chrono can handle extremely early years, HTTP date does not support
        // years before 0000
        let year = if year < 0 {
            panic!("negative years not supported")
        } else {
            year as u32
        };

        // Extract the individual digits from year
        push_digit(&mut out, (year / 1000) as u8);
        push_digit(&mut out, (year / 100 % 10) as u8);
        push_digit(&mut out, (year / 10 % 10) as u8);
        push_digit(&mut out, (year % 10) as u8);

        out.push(' ');

        let hour = structured.time().hour() as u8;

        // Extract the individual digits from hour
        push_digit(&mut out, hour / 10);
        push_digit(&mut out, hour % 10);

        out.push(':');

        // Extract the individual digits from minute
        let minute = structured.minute() as u8;
        push_digit(&mut out, minute / 10);
        push_digit(&mut out, minute % 10);

        out.push(':');

        let second = structured.second() as u8;
        push_digit(&mut out, second / 10);
        push_digit(&mut out, second % 10);

        // If non-zero nanos, push a 3-digit fractional second
        let nanos = structured.timestamp_subsec_nanos();
        if nanos != 0 {
            out.push('.');
            push_digit(&mut out, (nanos / (NANOS_PER_SECOND / 10)) as u8);
            push_digit(&mut out, (nanos / (NANOS_PER_SECOND / 100) % 10) as u8);
            push_digit(&mut out, (nanos / (NANOS_PER_SECOND / 1000) % 10) as u8);
        }

        out.push_str(" GMT");

        out
    }

    /// Parse an IMF-fixdate formatted date into an Instant
    ///
    /// This function has a few caveats:
    /// 1. It DOES NOT support the "deprecated" formats supported by HTTP date
    /// 2. It supports up to 3 digits of subsecond precision
    ///
    /// Ok: "Mon, 16 Dec 2019 23:48:18 GMT"
    /// Ok: "Mon, 16 Dec 2019 23:48:18.123 GMT"
    /// Ok: "Mon, 16 Dec 2019 23:48:18.12 GMT"
    /// Not Ok: "Mon, 16 Dec 2019 23:48:18.1234 GMT"
    pub(crate) fn parse(s: &str) -> Result<Instant, DateParseError> {
        if !s.is_ascii() {
            return Err(DateParseError::Invalid("not ascii"));
        }
        let x = s.trim().as_bytes();
        parse_imf_fixdate(x)
    }

    pub(crate) fn read(s: &str) -> Result<(Instant, &str), DateParseError> {
        if !s.is_ascii() {
            return Err(DateParseError::Invalid("Date must be valid ascii"));
        }
        let (first_date, rest) = match find_subsequence(s.as_bytes(), b" GMT") {
            // split_at is correct because we asserted that this date is only valid ASCII so the byte index is
            // the same as the char index
            Some(idx) => s.split_at(idx),
            None => return Err(DateParseError::Invalid("Date did not end in GMT")),
        };
        Ok((parse(first_date)?, rest))
    }

    fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        haystack
            .windows(needle.len())
            .position(|window| window == needle)
            .map(|idx| idx + needle.len())
    }

    fn parse_imf_fixdate(s: &[u8]) -> Result<Instant, DateParseError> {
        // Example: `Sun, 06 Nov 1994 08:49:37 GMT`
        if s.len() < 29
            || s.len() > 33
            || !s.ends_with(b" GMT")
            || s[16] != b' '
            || s[19] != b':'
            || s[22] != b':'
        {
            return Err(DateParseError::Invalid("incorrectly shaped string"));
        }
        let nanos: u32 = match &s[25] {
            b'.' => {
                // The date must end with " GMT", so read from the character after the `.`
                // to 4 from the end
                let fraction_slice = &s[26..s.len() - 4];
                if fraction_slice.len() > 3 {
                    // Only thousandths are supported
                    return Err(DateParseError::Invalid("too much precision"));
                }
                let fraction: u32 = parse_slice(fraction_slice)?;
                // We need to convert the fractional second to nanoseconds, so we need to scale
                // according the the number of decimals provided
                let multiplier = [10, 100, 1000];
                fraction * (NANOS_PER_SECOND / multiplier[fraction_slice.len() - 1])
            }
            b' ' => 0,
            _ => return Err(DateParseError::Invalid("incorrectly shaped string")),
        };

        let hours = parse_slice(&s[17..19])?;

        let minutes = parse_slice(&s[20..22])?;
        let seconds = parse_slice(&s[23..25])?;
        let time = NaiveTime::from_hms_nano(hours, minutes, seconds, nanos);
        let month = match &s[7..12] {
            b" Jan " => 1,
            b" Feb " => 2,
            b" Mar " => 3,
            b" Apr " => 4,
            b" May " => 5,
            b" Jun " => 6,
            b" Jul " => 7,
            b" Aug " => 8,
            b" Sep " => 9,
            b" Oct " => 10,
            b" Nov " => 11,
            b" Dec " => 12,
            _ => return Err(DateParseError::Invalid("invalid month")),
        };
        let year = parse_slice(&s[12..16])?;
        let day = parse_slice(&s[5..7])?;
        let date = NaiveDate::from_ymd(year, month, day);
        let datetime = NaiveDateTime::new(date, time);

        Ok(Instant::from_secs_and_nanos(
            datetime.timestamp(),
            datetime.timestamp_subsec_nanos(),
        ))
    }

    fn parse_slice<T>(ascii_slice: &[u8]) -> Result<T, DateParseError>
    where
        T: FromStr,
    {
        let as_str =
            std::str::from_utf8(ascii_slice).expect("should only be called on ascii strings");
        as_str
            .parse::<T>()
            .map_err(|_| DateParseError::IntParseError)
    }
}

#[cfg(test)]
mod test_http_date {
    use proptest::prelude::*;

    use crate::instant::format::{http_date, rfc3339, DateParseError};
    use crate::Instant;

    #[test]
    fn http_date_format() {
        let basic_http_date = "Mon, 16 Dec 2019 23:48:18 GMT";
        let ts = 1576540098;
        let instant = Instant::from_epoch_seconds(ts);
        assert_eq!(http_date::format(&instant), basic_http_date);
        assert_eq!(http_date::parse(basic_http_date), Ok(instant));
    }

    #[test]
    fn http_date_pre_epoch() {
        let pre_epoch = "Sat, 27 Jan 1962 20:40:12.120 GMT";
        let instant = Instant::from_secs_and_nanos(-250139988, 120_000_000);
        assert_eq!(http_date::parse(pre_epoch), Ok(instant));
        assert_eq!(http_date::format(&instant), pre_epoch);
    }

    #[test]
    fn http_date_format_fractional_zeroed() {
        let basic_http_date = "Mon, 16 Dec 2019 23:48:18 GMT";
        let fractional = "Mon, 16 Dec 2019 23:48:18.000 GMT";
        let ts = 1576540098;
        let instant = Instant::from_epoch_seconds(ts);
        assert_eq!(http_date::format(&instant), basic_http_date);
        assert_eq!(http_date::parse(fractional), Ok(instant));
    }

    #[test]
    fn http_date_format_fractional_nonzero() {
        let fractional = "Mon, 16 Dec 2019 23:48:18.12 GMT";
        let fractional_normalized = "Mon, 16 Dec 2019 23:48:18.120 GMT";
        let ts = 1576540098;
        let instant = Instant::from_fractional_seconds(ts, 0.12);
        assert_eq!(http_date::parse(fractional), Ok(instant));
        assert_eq!(http_date::format(&instant), fractional_normalized);
    }

    #[test]
    fn http_date_format_fractional_nonzero2() {
        let fractional = "Mon, 16 Dec 2019 23:48:18.123 GMT";
        let fractional_normalized = "Mon, 16 Dec 2019 23:48:18.123 GMT";
        let ts = 1576540098;
        let instant = Instant::from_fractional_seconds(ts, 0.123);
        assert_eq!(http_date::parse(fractional), Ok(instant));
        assert_eq!(http_date::format(&instant), fractional_normalized);
    }

    #[test]
    fn too_much_fraction() {
        let fractional = "Mon, 16 Dec 2019 23:48:18.1212 GMT";
        assert_eq!(
            http_date::parse(fractional),
            Err(DateParseError::Invalid("incorrectly shaped string"))
        );
    }

    #[test]
    fn no_fraction() {
        let fractional = "Mon, 16 Dec 2019 23:48:18. GMT";
        assert_eq!(
            http_date::parse(fractional),
            Err(DateParseError::IntParseError)
        );
    }

    #[test]
    fn read_date() {
        let fractional = "Mon, 16 Dec 2019 23:48:18.123 GMT,some more stuff";
        let ts = 1576540098;
        let expected = Instant::from_fractional_seconds(ts, 0.123);
        let (actual, rest) = http_date::read(fractional).expect("valid");
        assert_eq!(rest, ",some more stuff");
        assert_eq!(expected, actual);
        http_date::read(rest).expect_err("invalid date");
    }

    #[track_caller]
    fn check_roundtrip(epoch_secs: i64, subsecond_nanos: u32) {
        let instant = Instant::from_secs_and_nanos(epoch_secs, subsecond_nanos);
        let formatted = http_date::format(&instant);
        let parsed = http_date::parse(&formatted);
        let read = http_date::read(&formatted);
        match parsed {
            Err(failure) => panic!("Date failed to parse {:?}", failure),
            Ok(date) => {
                assert!(read.is_ok());
                if date.subsecond_nanos != subsecond_nanos {
                    assert_eq!(http_date::format(&instant), formatted);
                } else {
                    assert_eq!(date, instant)
                }
            }
        }
    }

    #[test]
    fn http_date_roundtrip() {
        for epoch_secs in -1000..1000 {
            check_roundtrip(epoch_secs, 1);
        }

        check_roundtrip(1576540098, 0);
        check_roundtrip(9999999999, 0);
    }

    #[test]
    fn valid_iso_date() {
        let date = "1985-04-12T23:20:50.52Z";
        let expected = Instant::from_secs_and_nanos(482196050, 520000000);
        assert_eq!(rfc3339::parse(date), Ok(expected));
    }

    #[test]
    fn iso_date_no_fractional() {
        let date = "1985-04-12T23:20:50Z";
        let expected = Instant::from_secs_and_nanos(482196050, 0);
        assert_eq!(rfc3339::parse(date), Ok(expected));
    }

    #[test]
    fn read_iso_date_comma_split() {
        let date = "1985-04-12T23:20:50Z,1985-04-12T23:20:51Z";
        let (e1, date) = rfc3339::read(date).expect("should succeed");
        let (e2, date2) = rfc3339::read(&date[1..]).expect("should succeed");
        assert_eq!(date2, "");
        assert_eq!(date, ",1985-04-12T23:20:51Z");
        let expected = Instant::from_secs_and_nanos(482196050, 0);
        assert_eq!(e1, expected);
        let expected = Instant::from_secs_and_nanos(482196051, 0);
        assert_eq!(e2, expected);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10000))]

        #[test]
        fn round_trip(secs in -10000000..9999999999i64, nanos in 0..1_000_000_000u32) {
            check_roundtrip(secs, nanos);
        }
    }
}

pub(crate) mod rfc3339 {
    use chrono::format;

    use crate::instant::format::DateParseError;
    use crate::Instant;
    use chrono::{Datelike, Timelike};

    // OK: 1985-04-12T23:20:50.52Z
    // OK: 1985-04-12T23:20:50Z
    //
    // Timezones not supported:
    // Not OK: 1985-04-12T23:20:50-02:00
    pub(crate) fn parse(s: &str) -> Result<Instant, DateParseError> {
        let mut date = format::Parsed::new();
        let format = format::StrftimeItems::new("%Y-%m-%dT%H:%M:%S%.fZ");
        // TODO: it may be helpful for debugging to keep these errors around
        chrono::format::parse(&mut date, s, format)
            .map_err(|_| DateParseError::Invalid("invalid rfc3339 date"))?;
        let utc_date = date
            .to_naive_datetime_with_offset(0)
            .map_err(|_| DateParseError::Invalid("invalid date"))?;
        Ok(Instant::from_secs_and_nanos(
            utc_date.timestamp(),
            utc_date.timestamp_subsec_nanos(),
        ))
    }

    /// Read 1 RFC-3339 date from &str and return the remaining str
    pub(crate) fn read(s: &str) -> Result<(Instant, &str), DateParseError> {
        let delim = s.find('Z').map(|idx| idx + 1).unwrap_or_else(|| s.len());
        let (head, rest) = s.split_at(delim);
        Ok((parse(head)?, &rest))
    }

    /// Format an [Instant] in the RFC-3339 date format
    pub(crate) fn format(instant: &Instant) -> String {
        use std::fmt::Write;
        let (year, month, day, hour, minute, second, nanos) = {
            let s = instant.to_chrono_internal();
            (
                s.year(),
                s.month(),
                s.day(),
                s.time().hour(),
                s.time().minute(),
                s.time().second(),
                s.timestamp_subsec_nanos(),
            )
        };

        // This is stated in the assumptions for RFC-3339. ISO-8601 allows for years
        // between -99,999 and 99,999 inclusive, but RFC-3339 is bound between 0 and 9,999.
        assert!(
            (0..=9_999).contains(&year),
            "years must be between 0 and 9,999 in RFC-3339"
        );

        let mut out = String::with_capacity(33);
        write!(
            out,
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
            year, month, day, hour, minute, second
        )
        .unwrap();
        format_subsecond_fraction(&mut out, nanos);
        out.push('Z');
        out
    }

    /// Formats sub-second fraction for RFC-3339 (including the '.').
    /// Expects to be called with a number of `nanos` between 0 and 999_999_999 inclusive.
    /// The formatted fraction will be truncated to microseconds.
    fn format_subsecond_fraction(into: &mut String, nanos: u32) {
        debug_assert!(nanos < 1_000_000_000);
        let micros = nanos / 1000;
        if micros > 0 {
            into.push('.');
            let (mut remaining, mut place) = (micros, 100_000);
            while remaining > 0 {
                let digit = (remaining / place) % 10;
                into.push(char::from(b'0' + (digit as u8)));
                remaining -= digit * place;
                place /= 10;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::rfc3339::format;
    use crate::Instant;
    use proptest::proptest;

    #[test]
    fn no_fraction() {
        assert_eq!(
            "1970-01-01T00:00:00Z",
            format(&Instant::from_epoch_seconds(0))
        );
        assert_eq!(
            "2021-06-09T23:17:26Z",
            format(&Instant::from_epoch_seconds(1623280646))
        );
        assert_eq!(
            "1969-12-31T18:22:50Z",
            format(&Instant::from_epoch_seconds(-20230))
        );
    }

    #[test]
    fn with_fraction() {
        assert_eq!(
            "1970-01-01T00:00:00.987Z",
            format(&Instant::from_secs_and_nanos(0, 987_000_000))
        );
        assert_eq!(
            "1970-01-01T00:00:00.1Z",
            format(&Instant::from_secs_and_nanos(0, 100_000_000))
        );
        assert_eq!(
            "1970-01-01T00:00:00.01Z",
            format(&Instant::from_secs_and_nanos(0, 10_000_000))
        );
        assert_eq!(
            "1970-01-01T00:00:00.001Z",
            format(&Instant::from_secs_and_nanos(0, 1_000_000))
        );
        assert_eq!(
            "1970-01-01T00:00:00.987654Z",
            format(&Instant::from_secs_and_nanos(0, 987_654_000))
        );
        assert_eq!(
            "1970-01-01T00:00:00.987654Z",
            format(&Instant::from_secs_and_nanos(0, 987_654_321))
        );
        assert_eq!(
            "1970-01-01T00:00:00.000001Z",
            format(&Instant::from_secs_and_nanos(0, 1_000))
        );
        assert_eq!(
            "1970-01-01T00:00:00Z",
            format(&Instant::from_secs_and_nanos(0, 1))
        );
        assert_eq!(
            "1970-01-01T00:00:00.101Z",
            format(&Instant::from_secs_and_nanos(0, 101_000_000))
        );
    }

    proptest! {
        // Sanity test against chrono
        #[test]
        #[cfg(feature = "chrono-conversions")]
        fn proptest_rfc3339(
            seconds in 0..253_402_300_799i64, // 0 to 9999-12-31T23:59:59
            micros in 0..1_000_000u32
        ) {
            use chrono::DateTime;

            let nanos = micros * 1000;
            let instant = Instant::from_secs_and_nanos(seconds, nanos);
            let formatted = format(&instant);
            let parsed: Instant = DateTime::parse_from_rfc3339(&formatted).unwrap().into();
            assert_eq!(instant, parsed);
        }
    }
}
