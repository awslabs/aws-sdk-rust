/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Conversions from [`DateTime`] to the types in the
//! [`jiff`](https://crates.io/crates/jiff),
//! [`time`](https://crates.io/crates/time), or
//! [`chrono`](https://crates.io/crates/chrono)
//! crates.

use aws_smithy_types::DateTime;
use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
enum ErrorKind {
    /// Conversion failed because the value being converted is out of range for its destination
    OutOfRange(Box<dyn StdError + Send + Sync + 'static>),
}

/// Conversion error
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    fn out_of_range(source: impl Into<Box<dyn StdError + Send + Sync + 'static>>) -> Self {
        Self {
            kind: ErrorKind::OutOfRange(source.into()),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match &self.kind {
            ErrorKind::OutOfRange(source) => Some(source.as_ref() as _),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ErrorKind::OutOfRange(_) => {
                write!(
                    f,
                    "conversion failed because the value is out of range for its destination",
                )
            }
        }
    }
}

/// Adds functions to [`DateTime`] to convert it to `jiff`, `time`, or `chrono` types.
///
#[cfg_attr(
    feature = "convert-jiff-1-x",
    doc = r##"
# Example with `jiff`

Make sure your `Cargo.toml` enables the `convert-jiff-1-x` feature:
```toml
[dependencies]
aws-smithy-types-convert = { version = "VERSION", features = ["convert-jiff-1-x"] }
```

Then import [`DateTimeExt`] to use the conversions:
```rust
# use aws_smithy_types::DateTime;
use aws_smithy_types_convert::date_time::DateTimeExt;
use jiff::Timestamp;

let timestamp: Timestamp = DateTime::from_secs(5).to_jiff()?;
let date_time: DateTime = DateTime::from_jiff(timestamp);
# Ok::<(), aws_smithy_types_convert::date_time::Error>(())
```
"##
)]
#[cfg_attr(
    feature = "convert-time",
    doc = r##"
# Example with `time`

Make sure your **Cargo.toml** enables the `convert-time` feature:
```toml
[dependencies]
aws-smithy-types-convert = { version = "VERSION", features = ["convert-time"] }
```

Then import [`DateTimeExt`] to use the conversions:
```rust
# fn test_fn() -> Result<(), aws_smithy_types_convert::date_time::Error> {
# use aws_smithy_types::DateTime;
use aws_smithy_types_convert::date_time::DateTimeExt;
use time::OffsetDateTime;

let offset_date_time: OffsetDateTime = DateTime::from_secs(5).to_time()?;
let date_time: DateTime  = DateTime::from_time(offset_date_time);
# Ok(())
# }
```
"##
)]
#[cfg_attr(
    feature = "convert-chrono",
    doc = r##"
# Example with `chrono`

Make sure your **Cargo.toml** enables the `convert-chrono` feature:
```toml
[dependencies]
aws-smithy-types-convert = { version = "VERSION", features = ["convert-chrono"] }
```

Then import [`DateTimeExt`] to use the conversions:
```rust
# use aws_smithy_types::DateTime;
use aws_smithy_types_convert::date_time::DateTimeExt;
use chrono::{Utc};

let chrono_date_time: chrono::DateTime<Utc> = DateTime::from_secs(5).to_chrono_utc().unwrap();
let date_time: DateTime = DateTime::from_chrono_utc(chrono_date_time);
```
"##
)]
pub trait DateTimeExt {
    /// Converts a [`DateTime`] to a [`jiff::Timestamp`].
    ///
    /// Returns an [`Error`] if the time is after [`jiff::Timestamp::MAX`]
    /// (`9999-12-30T22:00:00.999999999Z`) or before [`jiff::Timestamp::MIN`]
    /// (`-009999-01-02T01:59:59Z`).
    #[cfg(feature = "convert-jiff-1-x")]
    fn to_jiff(&self) -> Result<jiff::Timestamp, Error>;

    /// Converts a [`jiff::Timestamp`] to a [`DateTime`].
    #[cfg(feature = "convert-jiff-1-x")]
    fn from_jiff(timestamp: jiff::Timestamp) -> DateTime;

    /// Converts a [`DateTime`] to a [`chrono::DateTime`] with timezone UTC.
    #[cfg(feature = "convert-chrono")]
    fn to_chrono_utc(&self) -> Result<chrono::DateTime<chrono::Utc>, Error>;

    /// Converts a [`chrono::DateTime`] with timezone UTC to a [`DateTime`].
    #[cfg(feature = "convert-chrono")]
    fn from_chrono_utc(time: chrono::DateTime<chrono::Utc>) -> DateTime;

    /// Converts a [`chrono::DateTime`] with an offset timezone to a [`DateTime`].
    #[cfg(feature = "convert-chrono")]
    fn from_chrono_fixed(time: chrono::DateTime<chrono::FixedOffset>) -> DateTime;

    /// Converts a [`DateTime`] to a [`time::OffsetDateTime`].
    ///
    /// Returns an [`Error`] if the time is after
    /// `9999-12-31T23:59:59.999Z` or before `-9999-01-01T00:00:00.000Z`.
    #[cfg(feature = "convert-time")]
    fn to_time(&self) -> Result<time::OffsetDateTime, Error>;

    /// Converts a [`time::OffsetDateTime`] to a [`DateTime`].
    #[cfg(feature = "convert-time")]
    fn from_time(time: time::OffsetDateTime) -> DateTime;
}

impl DateTimeExt for DateTime {
    #[cfg(feature = "convert-jiff-1-x")]
    fn to_jiff(&self) -> Result<jiff::Timestamp, Error> {
        jiff::Timestamp::from_nanosecond(self.as_nanos()).map_err(Error::out_of_range)
    }

    #[cfg(feature = "convert-jiff-1-x")]
    fn from_jiff(timestamp: jiff::Timestamp) -> DateTime {
        DateTime::from_nanos(timestamp.as_nanosecond())
            .expect("DateTime supports a greater range than Timestamp")
    }

    #[cfg(feature = "convert-chrono")]
    fn to_chrono_utc(&self) -> Result<chrono::DateTime<chrono::Utc>, Error> {
        match chrono::DateTime::from_timestamp(self.secs(), self.subsec_nanos()) {
            None => Err(Error::out_of_range(format!(
                "out-of-range seconds {} or invalid nanoseconds {}",
                self.secs(),
                self.subsec_nanos()
            ))),
            Some(dt) => Ok(dt),
        }
    }

    #[cfg(feature = "convert-chrono")]
    fn from_chrono_utc(value: chrono::DateTime<chrono::Utc>) -> DateTime {
        DateTime::from_secs_and_nanos(value.timestamp(), value.timestamp_subsec_nanos())
    }

    #[cfg(feature = "convert-chrono")]
    fn from_chrono_fixed(value: chrono::DateTime<chrono::FixedOffset>) -> DateTime {
        Self::from_chrono_utc(value.with_timezone(&chrono::Utc))
    }

    #[cfg(feature = "convert-time")]
    fn to_time(&self) -> Result<time::OffsetDateTime, Error> {
        time::OffsetDateTime::from_unix_timestamp_nanos(self.as_nanos()).map_err(|err| Error {
            kind: ErrorKind::OutOfRange(err.into()),
        })
    }

    #[cfg(feature = "convert-time")]
    fn from_time(time: time::OffsetDateTime) -> DateTime {
        DateTime::from_nanos(time.unix_timestamp_nanos())
            .expect("DateTime supports a greater range than OffsetDateTime")
    }
}

#[cfg(all(
    test,
    any(
        feature = "convert-chrono",
        feature = "convert-jiff-1-x",
        feature = "convert-time"
    )
))]
mod test {
    use super::DateTimeExt;
    use aws_smithy_types::date_time::{DateTime, Format};
    #[cfg(feature = "convert-chrono")]
    use chrono::Timelike;

    #[cfg(any(feature = "convert-jiff-1-x", feature = "convert-time"))]
    use super::{Error, ErrorKind};

    #[test]
    #[cfg(feature = "convert-jiff-1-x")]
    fn from_jiff() {
        use jiff::Timestamp;

        let timestamp: Timestamp = "2039-07-08T09:03:11.123Z".parse().unwrap();
        let expected = DateTime::from_str("2039-07-08T09:03:11.123Z", Format::DateTime).unwrap();
        assert_eq!(expected, DateTime::from_jiff(timestamp));

        let timestamp: Timestamp = "1000-07-08T09:03:11.456Z".parse().unwrap();
        let expected = DateTime::from_str("1000-07-08T09:03:11.456Z", Format::DateTime).unwrap();
        assert_eq!(expected, DateTime::from_jiff(timestamp));

        // Before the epoch, with subsecond precision
        let timestamp = Timestamp::new(-2, 500_000_000).unwrap();
        assert_eq!(
            DateTime::from_secs_and_nanos(-2, 500_000_000),
            DateTime::from_jiff(timestamp)
        );

        for timestamp in [Timestamp::MIN, Timestamp::MAX] {
            assert_eq!(timestamp, DateTime::from_jiff(timestamp).to_jiff().unwrap());
        }
    }

    #[test]
    #[cfg(feature = "convert-jiff-1-x")]
    fn to_jiff() {
        use jiff::Timestamp;

        let date_time = DateTime::from_str("2039-07-08T09:03:11.123Z", Format::DateTime).unwrap();
        let expected: Timestamp = "2039-07-08T09:03:11.123Z".parse().unwrap();
        assert_eq!(expected, date_time.to_jiff().unwrap());

        let date_time = DateTime::from_str("1000-07-08T09:03:11.456Z", Format::DateTime).unwrap();
        let expected: Timestamp = "1000-07-08T09:03:11.456Z".parse().unwrap();
        assert_eq!(expected, date_time.to_jiff().unwrap());

        let date_time = DateTime::from_secs_and_nanos(-2, 500_000_000);
        assert_eq!(
            Timestamp::new(-2, 500_000_000).unwrap(),
            date_time.to_jiff().unwrap()
        );

        // `set_subsec_nanos` doesn't limit the nanos to less than one second
        let mut date_time = DateTime::from_secs(10);
        date_time.set_subsec_nanos(u32::MAX);
        assert_eq!(
            Timestamp::new(14, 294_967_295).unwrap(),
            date_time.to_jiff().unwrap()
        );

        // The limits named in the docs of `to_jiff`
        assert_eq!("9999-12-30T22:00:00.999999999Z", Timestamp::MAX.to_string());
        assert_eq!("-009999-01-02T01:59:59Z", Timestamp::MIN.to_string());

        for out_of_range in [
            DateTime::from_secs(i64::MAX),
            DateTime::from_secs(i64::MIN),
            DateTime::from_nanos(Timestamp::MAX.as_nanosecond() + 1).unwrap(),
            DateTime::from_nanos(Timestamp::MIN.as_nanosecond() - 1).unwrap(),
        ] {
            let date_time = out_of_range;
            assert!(matches!(
                date_time.to_jiff(),
                Err(Error {
                    kind: ErrorKind::OutOfRange(_)
                })
            ));
        }
    }

    #[test]
    #[cfg(feature = "convert-chrono")]
    fn from_chrono() {
        use chrono::{FixedOffset, TimeZone, Utc};

        let chrono = Utc
            .with_ymd_and_hms(2039, 7, 8, 9, 3, 11)
            .unwrap()
            .with_nanosecond(123_000_000)
            .unwrap();
        let expected = DateTime::from_str("2039-07-08T09:03:11.123Z", Format::DateTime).unwrap();
        assert_eq!(expected, DateTime::from_chrono_utc(chrono));

        let chrono = Utc
            .with_ymd_and_hms(1000, 7, 8, 9, 3, 11)
            .unwrap()
            .with_nanosecond(456_000_000)
            .unwrap();
        let expected = DateTime::from_str("1000-07-08T09:03:11.456Z", Format::DateTime).unwrap();
        assert_eq!(expected, DateTime::from_chrono_utc(chrono));

        let chrono = FixedOffset::west_opt(2 * 3600)
            .unwrap()
            .with_ymd_and_hms(2039, 7, 8, 9, 3, 11)
            .unwrap()
            .with_nanosecond(123_000_000)
            .unwrap();
        let expected = DateTime::from_str("2039-07-08T11:03:11.123Z", Format::DateTime).unwrap();
        assert_eq!(expected, DateTime::from_chrono_fixed(chrono));
    }

    #[test]
    #[cfg(feature = "convert-chrono")]
    fn to_chrono() {
        use chrono::{TimeZone, Utc};

        let date_time = DateTime::from_str("2039-07-08T09:03:11.123Z", Format::DateTime).unwrap();
        let expected = Utc
            .with_ymd_and_hms(2039, 7, 8, 9, 3, 11)
            .unwrap()
            .with_nanosecond(123_000_000)
            .unwrap();
        assert_eq!(expected, date_time.to_chrono_utc().unwrap());

        let date_time = DateTime::from_str("1000-07-08T09:03:11.456Z", Format::DateTime).unwrap();
        let expected = Utc
            .with_ymd_and_hms(1000, 7, 8, 9, 3, 11)
            .unwrap()
            .with_nanosecond(456_000_000)
            .unwrap();
        assert_eq!(expected, date_time.to_chrono_utc().unwrap());
    }

    #[test]
    #[cfg(feature = "convert-time")]
    fn from_time() {
        use time::{Date, Month, PrimitiveDateTime, Time};

        let time = PrimitiveDateTime::new(
            Date::from_calendar_date(2039, Month::July, 8).unwrap(),
            Time::from_hms_milli(9, 3, 11, 123).unwrap(),
        )
        .assume_utc();
        let expected = DateTime::from_str("2039-07-08T09:03:11.123Z", Format::DateTime).unwrap();
        assert_eq!(expected, DateTime::from_time(time));

        let time = PrimitiveDateTime::new(
            Date::from_calendar_date(1000, Month::July, 8).unwrap(),
            Time::from_hms_milli(9, 3, 11, 456).unwrap(),
        )
        .assume_utc();
        let expected = DateTime::from_str("1000-07-08T09:03:11.456Z", Format::DateTime).unwrap();
        assert_eq!(expected, DateTime::from_time(time));
    }

    #[test]
    #[cfg(feature = "convert-time")]
    fn to_time() {
        use time::{Date, Month, PrimitiveDateTime, Time};

        let date_time = DateTime::from_str("2039-07-08T09:03:11.123Z", Format::DateTime).unwrap();
        let expected = PrimitiveDateTime::new(
            Date::from_calendar_date(2039, Month::July, 8).unwrap(),
            Time::from_hms_milli(9, 3, 11, 123).unwrap(),
        )
        .assume_utc();
        assert_eq!(expected, date_time.to_time().unwrap());

        let date_time = DateTime::from_str("1000-07-08T09:03:11.456Z", Format::DateTime).unwrap();
        let expected = PrimitiveDateTime::new(
            Date::from_calendar_date(1000, Month::July, 8).unwrap(),
            Time::from_hms_milli(9, 3, 11, 456).unwrap(),
        )
        .assume_utc();
        assert_eq!(expected, date_time.to_time().unwrap());

        let date_time = DateTime::from_secs_and_nanos(i64::MAX, 0);
        assert!(matches!(
            date_time.to_time(),
            Err(Error {
                kind: ErrorKind::OutOfRange(_)
            })
        ));
        let date_time = DateTime::from_secs_and_nanos(i64::MIN, 0);
        assert!(matches!(
            date_time.to_time(),
            Err(Error {
                kind: ErrorKind::OutOfRange(_)
            })
        ));
    }
}
