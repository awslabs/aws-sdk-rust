/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use crate::instant::format::DateParseError;
use chrono::{DateTime, NaiveDateTime, Utc};
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

mod format;

/* ANCHOR: instant */

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Instant {
    seconds: i64,
    subsecond_nanos: u32,
}

/* ANCHOR_END: instant */

impl Instant {
    pub fn from_epoch_seconds(epoch_seconds: i64) -> Self {
        Instant {
            seconds: epoch_seconds,
            subsecond_nanos: 0,
        }
    }

    pub fn from_fractional_seconds(epoch_seconds: i64, fraction: f64) -> Self {
        let subsecond_nanos = (fraction * 1_000_000_000_f64) as u32;
        Instant::from_secs_and_nanos(epoch_seconds, subsecond_nanos)
    }

    pub fn from_secs_and_nanos(seconds: i64, subsecond_nanos: u32) -> Self {
        if subsecond_nanos >= 1_000_000_000 {
            panic!("{} is > 1_000_000_000", subsecond_nanos)
        }
        Instant {
            seconds,
            subsecond_nanos,
        }
    }

    pub fn from_f64(epoch_seconds: f64) -> Self {
        let seconds = epoch_seconds.floor() as i64;
        let rem = epoch_seconds - epoch_seconds.floor();
        Instant::from_fractional_seconds(seconds, rem)
    }

    pub fn from_system_time(system_time: SystemTime) -> Self {
        let duration = system_time
            .duration_since(UNIX_EPOCH)
            .expect("SystemTime can never represent a time before the Unix Epoch");
        Instant {
            seconds: duration.as_secs() as i64,
            subsecond_nanos: duration.subsec_nanos(),
        }
    }

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

    #[cfg(feature = "chrono-conversions")]
    pub fn to_chrono(&self) -> DateTime<Utc> {
        self.to_chrono_internal()
    }

    fn to_chrono_internal(&self) -> DateTime<Utc> {
        DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp(self.seconds, self.subsecond_nanos),
            Utc,
        )
    }

    /// Convert this `Instant` to a [`SystemTime`](std::time::SystemTime)
    ///
    /// Since SystemTime cannot represent times prior to the unix epoch, if this time is before
    /// 1/1/1970, this function will return `None`.
    pub fn to_system_time(&self) -> Option<SystemTime> {
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

    pub fn has_nanos(&self) -> bool {
        self.subsecond_nanos != 0
    }

    pub fn epoch_fractional_seconds(&self) -> f64 {
        self.seconds as f64 + self.subsecond_nanos as f64 / 1_000_000_000_f64
    }

    pub fn epoch_seconds(&self) -> i64 {
        self.seconds
    }

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

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Format {
    DateTime,
    HttpDate,
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
        let instant = Instant::from_secs_and_nanos(1234, 56789);
        let chrono = instant.to_chrono();
        let instant_again: Instant = chrono.into();
        assert_eq!(instant, instant_again);
    }
}
