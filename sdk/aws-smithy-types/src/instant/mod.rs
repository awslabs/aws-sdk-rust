/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Instant value for representing Smithy timestamps.
//!
//! Unlike [`std::time::Instant`], this instant is not opaque. The time inside of it can be
//! read and modified. It also holds logic for parsing and formatting timestamps in any of
//! the timestamp formats that [Smithy](https://awslabs.github.io/smithy/) supports.

use crate::instant::format::DateParseError;
use chrono::{DateTime, NaiveDateTime, Utc};
use num_integer::div_mod_floor;
use num_integer::Integer;
use std::error::Error as StdError;
use std::fmt;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

mod format;

const MILLIS_PER_SECOND: i64 = 1000;
const NANOS_PER_MILLI: u32 = 1_000_000;

/* ANCHOR: instant */

/// Instant in time.
///
/// Instant in time represented as seconds and sub-second nanos since
/// the Unix epoch (January 1, 1970 at midnight UTC/GMT).
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Instant {
    seconds: i64,
    subsecond_nanos: u32,
}

/* ANCHOR_END: instant */

impl Instant {
    /// Creates an `Instant` from a number of seconds since the Unix epoch.
    pub fn from_epoch_seconds(epoch_seconds: i64) -> Self {
        Instant {
            seconds: epoch_seconds,
            subsecond_nanos: 0,
        }
    }

    /// Creates an `Instant` from a number of seconds and a fractional second since the Unix epoch.
    ///
    /// # Example
    /// ```
    /// # use aws_smithy_types::Instant;
    /// assert_eq!(
    ///     Instant::from_secs_and_nanos(1, 500_000_000u32),
    ///     Instant::from_fractional_seconds(1, 0.5),
    /// );
    /// ```
    pub fn from_fractional_seconds(epoch_seconds: i64, fraction: f64) -> Self {
        let subsecond_nanos = (fraction * 1_000_000_000_f64) as u32;
        Instant::from_secs_and_nanos(epoch_seconds, subsecond_nanos)
    }

    /// Creates an `Instant` from a number of seconds and sub-second nanos since the Unix epoch.
    ///
    /// # Example
    /// ```
    /// # use aws_smithy_types::Instant;
    /// assert_eq!(
    ///     Instant::from_fractional_seconds(1, 0.5),
    ///     Instant::from_secs_and_nanos(1, 500_000_000u32),
    /// );
    /// ```
    pub fn from_secs_and_nanos(seconds: i64, subsecond_nanos: u32) -> Self {
        if subsecond_nanos >= 1_000_000_000 {
            panic!("{} is > 1_000_000_000", subsecond_nanos)
        }
        Instant {
            seconds,
            subsecond_nanos,
        }
    }

    /// Creates an `Instant` from an `f64` representing the number of seconds since the Unix epoch.
    ///
    /// # Example
    /// ```
    /// # use aws_smithy_types::Instant;
    /// assert_eq!(
    ///     Instant::from_fractional_seconds(1, 0.5),
    ///     Instant::from_f64(1.5),
    /// );
    /// ```
    pub fn from_f64(epoch_seconds: f64) -> Self {
        let seconds = epoch_seconds.floor() as i64;
        let rem = epoch_seconds - epoch_seconds.floor();
        Instant::from_fractional_seconds(seconds, rem)
    }

    /// Creates an `Instant` from a [`SystemTime`].
    pub fn from_system_time(system_time: SystemTime) -> Self {
        let duration = system_time
            .duration_since(UNIX_EPOCH)
            .expect("SystemTime can never represent a time before the Unix Epoch");
        Instant {
            seconds: duration.as_secs() as i64,
            subsecond_nanos: duration.subsec_nanos(),
        }
    }

    /// Parses an `Instant` from a string using the given `format`.
    pub fn from_str(s: &str, format: Format) -> Result<Self, DateParseError> {
        match format {
            Format::DateTime => format::rfc3339::parse(s),
            Format::HttpDate => format::http_date::parse(s),
            Format::EpochSeconds => <f64>::from_str(s)
                // TODO: Parse base & fraction separately to achieve higher precision
                .map(Self::from_f64)
                .map_err(|_| DateParseError::Invalid("expected float")),
        }
    }

    /// Read 1 date of `format` from `s`, expecting either `delim` or EOF
    ///
    /// Enable parsing multiple dates from the same string
    pub fn read(s: &str, format: Format, delim: char) -> Result<(Self, &str), DateParseError> {
        let (inst, next) = match format {
            Format::DateTime => format::rfc3339::read(s)?,
            Format::HttpDate => format::http_date::read(s)?,
            Format::EpochSeconds => {
                let split_point = s.find(delim).unwrap_or_else(|| s.len());
                let (s, rest) = s.split_at(split_point);
                (Self::from_str(s, format)?, rest)
            }
        };
        if next.is_empty() {
            Ok((inst, next))
        } else if next.starts_with(delim) {
            Ok((inst, &next[1..]))
        } else {
            Err(DateParseError::Invalid("didn't find expected delimiter"))
        }
    }

    /// Converts the `Instant` to a chrono `DateTime<Utc>`.
    #[cfg(feature = "chrono-conversions")]
    pub fn to_chrono(self) -> DateTime<Utc> {
        self.to_chrono_internal()
    }

    fn to_chrono_internal(self) -> DateTime<Utc> {
        DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp(self.seconds, self.subsecond_nanos),
            Utc,
        )
    }

    /// Convert this `Instant` to a [`SystemTime`](std::time::SystemTime)
    ///
    /// Since SystemTime cannot represent times prior to the unix epoch, if this time is before
    /// 1/1/1970, this function will return `None`.
    pub fn to_system_time(self) -> Option<SystemTime> {
        if self.seconds < 0 {
            None
        } else {
            Some(
                UNIX_EPOCH
                    + Duration::from_secs(self.seconds as u64)
                    + Duration::from_nanos(self.subsecond_nanos as u64),
            )
        }
    }

    /// Returns true if sub-second nanos is greater than zero.
    pub fn has_nanos(&self) -> bool {
        self.subsecond_nanos != 0
    }

    /// Returns the `Instant` value as an `f64` representing the seconds since the Unix epoch.
    pub fn epoch_fractional_seconds(&self) -> f64 {
        self.seconds as f64 + self.subsecond_nanos as f64 / 1_000_000_000_f64
    }

    /// Returns the epoch seconds component of the `Instant`.
    ///
    /// _Note: this does not include the sub-second nanos._
    pub fn epoch_seconds(&self) -> i64 {
        self.seconds
    }

    /// Returns the sub-second nanos component of the `Instant`.
    ///
    /// _Note: this does not include the number of seconds since the epoch._
    pub fn epoch_subsecond_nanos(&self) -> u32 {
        self.subsecond_nanos
    }

    /// Converts the `Instant` to the number of milliseconds since the Unix epoch.
    /// This is fallible since `Instant` holds more precision than an `i64`, and will
    /// return a `ConversionError` for `Instant` values that can't be converted.
    pub fn to_epoch_millis(self) -> Result<i64, ConversionError> {
        let subsec_millis =
            Integer::div_floor(&i64::from(self.subsecond_nanos), &(NANOS_PER_MILLI as i64));
        if self.seconds < 0 {
            self.seconds
                .checked_add(1)
                .and_then(|seconds| seconds.checked_mul(MILLIS_PER_SECOND))
                .and_then(|millis| millis.checked_sub(1000 - subsec_millis))
        } else {
            self.seconds
                .checked_mul(MILLIS_PER_SECOND)
                .and_then(|millis| millis.checked_add(subsec_millis))
        }
        .ok_or(ConversionError(
            "Instant value too large to fit into i64 epoch millis",
        ))
    }

    /// Converts number of milliseconds since the Unix epoch into an `Instant`.
    pub fn from_epoch_millis(epoch_millis: i64) -> Instant {
        let (seconds, millis) = div_mod_floor(epoch_millis, MILLIS_PER_SECOND);
        Instant::from_secs_and_nanos(seconds, millis as u32 * NANOS_PER_MILLI)
    }

    /// Formats the `Instant` to a string using the given `format`.
    pub fn fmt(&self, format: Format) -> String {
        match format {
            Format::DateTime => format::rfc3339::format(&self),
            Format::EpochSeconds => {
                if self.subsecond_nanos == 0 {
                    format!("{}", self.seconds)
                } else {
                    let fraction = format!("{:0>9}", self.subsecond_nanos);
                    format!("{}.{}", self.seconds, fraction.trim_end_matches('0'))
                }
            }
            Format::HttpDate => format::http_date::format(&self),
        }
    }
}

/// Failure to convert an `Instant` to or from another type.
#[derive(Debug)]
#[non_exhaustive]
pub struct ConversionError(&'static str);

impl StdError for ConversionError {}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "chrono-conversions")]
impl From<DateTime<Utc>> for Instant {
    fn from(value: DateTime<Utc>) -> Instant {
        Instant::from_secs_and_nanos(value.timestamp(), value.timestamp_subsec_nanos())
    }
}

#[cfg(feature = "chrono-conversions")]
impl From<DateTime<chrono::FixedOffset>> for Instant {
    fn from(value: DateTime<chrono::FixedOffset>) -> Instant {
        value.with_timezone(&Utc).into()
    }
}

/// Formats for representing an `Instant` in the Smithy protocols.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Format {
    /// RFC-3339 Date Time.
    DateTime,
    /// Date format used by the HTTP `Date` header, specified in RFC-7231.
    HttpDate,
    /// Number of seconds since the Unix epoch formatted as a floating point.
    EpochSeconds,
}

#[cfg(test)]
mod test {
    use crate::instant::Format;
    use crate::Instant;

    #[test]
    fn test_instant_fmt() {
        let instant = Instant::from_epoch_seconds(1576540098);
        assert_eq!(instant.fmt(Format::DateTime), "2019-12-16T23:48:18Z");
        assert_eq!(instant.fmt(Format::EpochSeconds), "1576540098");
        assert_eq!(
            instant.fmt(Format::HttpDate),
            "Mon, 16 Dec 2019 23:48:18 GMT"
        );

        let instant = Instant::from_fractional_seconds(1576540098, 0.52);
        assert_eq!(instant.fmt(Format::DateTime), "2019-12-16T23:48:18.52Z");
        assert_eq!(instant.fmt(Format::EpochSeconds), "1576540098.52");
        assert_eq!(
            instant.fmt(Format::HttpDate),
            "Mon, 16 Dec 2019 23:48:18.520 GMT"
        );
    }

    #[test]
    fn test_instant_fmt_zero_seconds() {
        let instant = Instant::from_epoch_seconds(1576540080);
        assert_eq!(instant.fmt(Format::DateTime), "2019-12-16T23:48:00Z");
        assert_eq!(instant.fmt(Format::EpochSeconds), "1576540080");
        assert_eq!(
            instant.fmt(Format::HttpDate),
            "Mon, 16 Dec 2019 23:48:00 GMT"
        );
    }

    #[test]
    fn test_read_single_http_date() {
        let s = "Mon, 16 Dec 2019 23:48:18 GMT";
        let (_, next) = Instant::read(s, Format::HttpDate, ',').expect("valid");
        assert_eq!(next, "");
    }

    #[test]
    fn test_read_single_float() {
        let s = "1576540098.52";
        let (_, next) = Instant::read(s, Format::EpochSeconds, ',').expect("valid");
        assert_eq!(next, "");
    }

    #[test]
    fn test_read_many_float() {
        let s = "1576540098.52,1576540098.53";
        let (_, next) = Instant::read(s, Format::EpochSeconds, ',').expect("valid");
        assert_eq!(next, "1576540098.53");
    }

    #[test]
    fn test_ready_many_http_date() {
        let s = "Mon, 16 Dec 2019 23:48:18 GMT,Tue, 17 Dec 2019 23:48:18 GMT";
        let (_, next) = Instant::read(s, Format::HttpDate, ',').expect("valid");
        assert_eq!(next, "Tue, 17 Dec 2019 23:48:18 GMT");
    }

    #[test]
    #[cfg(feature = "chrono-conversions")]
    fn chrono_conversions_round_trip() {
        for (seconds, nanos) in &[(1234, 56789), (-1234, 4321)] {
            let instant = Instant::from_secs_and_nanos(*seconds, *nanos);
            let chrono = instant.to_chrono();
            let instant_again: Instant = chrono.into();
            assert_eq!(instant, instant_again);
        }
    }

    #[derive(Debug)]
    struct EpochMillisTestCase {
        rfc3339: &'static str,
        epoch_millis: i64,
        epoch_seconds: i64,
        epoch_subsec_nanos: u32,
    }

    // These test case values were generated from the following Kotlin JVM code:
    // ```kotlin
    // val instant = Instant.ofEpochMilli(<epoch milli value>);
    // println(DateTimeFormatter.ISO_DATE_TIME.format(instant.atOffset(ZoneOffset.UTC)))
    // println(instant.epochSecond)
    // println(instant.nano)
    // ```
    const EPOCH_MILLIS_TEST_CASES: &[EpochMillisTestCase] = &[
        EpochMillisTestCase {
            rfc3339: "2021-07-30T21:20:04.123Z",
            epoch_millis: 1627680004123,
            epoch_seconds: 1627680004,
            epoch_subsec_nanos: 123000000,
        },
        EpochMillisTestCase {
            rfc3339: "1918-06-04T02:39:55.877Z",
            epoch_millis: -1627680004123,
            epoch_seconds: -1627680005,
            epoch_subsec_nanos: 877000000,
        },
        EpochMillisTestCase {
            rfc3339: "+292278994-08-17T07:12:55.807Z",
            epoch_millis: i64::MAX,
            epoch_seconds: 9223372036854775,
            epoch_subsec_nanos: 807000000,
        },
        EpochMillisTestCase {
            rfc3339: "-292275055-05-16T16:47:04.192Z",
            epoch_millis: i64::MIN,
            epoch_seconds: -9223372036854776,
            epoch_subsec_nanos: 192000000,
        },
    ];

    #[test]
    fn to_epoch_millis() {
        for test_case in EPOCH_MILLIS_TEST_CASES {
            println!("Test case: {:?}", test_case);
            let instant =
                Instant::from_secs_and_nanos(test_case.epoch_seconds, test_case.epoch_subsec_nanos);
            assert_eq!(test_case.epoch_seconds, instant.epoch_seconds());
            assert_eq!(
                test_case.epoch_subsec_nanos,
                instant.epoch_subsecond_nanos()
            );
            assert_eq!(test_case.epoch_millis, instant.to_epoch_millis().unwrap());
        }

        assert!(Instant::from_secs_and_nanos(i64::MAX, 0)
            .to_epoch_millis()
            .is_err());
    }

    #[test]
    fn from_epoch_millis() {
        for test_case in EPOCH_MILLIS_TEST_CASES {
            println!("Test case: {:?}", test_case);
            let instant = Instant::from_epoch_millis(test_case.epoch_millis);
            assert_eq!(test_case.epoch_seconds, instant.epoch_seconds());
            assert_eq!(
                test_case.epoch_subsec_nanos,
                instant.epoch_subsecond_nanos()
            );
        }
    }

    #[test]
    fn to_from_epoch_millis_round_trip() {
        for millis in &[0, 1627680004123, -1627680004123, i64::MAX, i64::MIN] {
            assert_eq!(
                *millis,
                Instant::from_epoch_millis(*millis)
                    .to_epoch_millis()
                    .unwrap()
            );
        }
    }
}
