/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Conversions from [`DateTime`] to the types in the
//! [`time`](https://crates.io/crates/time) or
//! [`chrono`](https://crates.io/crates/chrono)
//! crates.

use aws_smithy_types::DateTime;
use std::error::Error as StdError;
use std::fmt;

/// Conversion error
#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    /// Conversion failed because the value being converted is out of range for its destination
    #[non_exhaustive]
    OutOfRange(Box<dyn StdError + Send + Sync + 'static>),
}

impl StdError for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfRange(cause) => {
                write!(
                    f,
                    "conversion failed because the value is out of range for its destination: {}",
                    cause
                )
            }
        }
    }
}

/// Adds functions to [`DateTime`] to convert it to `time` or `chrono` types.
///
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

let chrono_date_time: chrono::DateTime<Utc> = DateTime::from_secs(5).to_chrono_utc();
let date_time: DateTime = DateTime::from_chrono_utc(chrono_date_time);
```
"##
)]
pub trait DateTimeExt {
    /// Converts a [`DateTime`] to a [`chrono::DateTime`] with timezone UTC.
    #[cfg(feature = "convert-chrono")]
    fn to_chrono_utc(&self) -> chrono::DateTime<chrono::Utc>;

    /// Converts a [`chrono::DateTime`] with timezone UTC to a [`DateTime`].
    #[cfg(feature = "convert-chrono")]
    fn from_chrono_utc(time: chrono::DateTime<chrono::Utc>) -> DateTime;

    /// Converts a [`chrono::DateTime`] with an offset timezone to a [`DateTime`].
    #[cfg(feature = "convert-chrono")]
    fn from_chrono_fixed(time: chrono::DateTime<chrono::FixedOffset>) -> DateTime;

    /// Converts a [`DateTime`] to a [`time::OffsetDateTime`].
    ///
    /// Returns an [`Error::OutOfRange`] if the time is after
    /// `9999-12-31T23:59:59.999Z` or before `-9999-01-01T00:00:00.000Z`.
    #[cfg(feature = "convert-time")]
    fn to_time(&self) -> Result<time::OffsetDateTime, Error>;

    /// Converts a [`time::OffsetDateTime`] to a [`DateTime`].
    #[cfg(feature = "convert-time")]
    fn from_time(time: time::OffsetDateTime) -> DateTime;
}

impl DateTimeExt for DateTime {
    #[cfg(feature = "convert-chrono")]
    fn to_chrono_utc(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::<chrono::Utc>::from_utc(
            chrono::NaiveDateTime::from_timestamp(self.secs(), self.subsec_nanos()),
            chrono::Utc,
        )
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
        time::OffsetDateTime::from_unix_timestamp_nanos(self.as_nanos())
            .map_err(|err| Error::OutOfRange(err.into()))
    }

    #[cfg(feature = "convert-time")]
    fn from_time(time: time::OffsetDateTime) -> DateTime {
        DateTime::from_nanos(time.unix_timestamp_nanos())
            .expect("DateTime supports a greater range than OffsetDateTime")
    }
}

#[cfg(all(test, any(feature = "convert-chrono", feature = "convert-time")))]
mod test {
    use super::DateTimeExt;
    use aws_smithy_types::date_time::{DateTime, Format};

    #[cfg(feature = "convert-time")]
    use super::Error;

    #[test]
    #[cfg(feature = "convert-chrono")]
    fn from_chrono() {
        use chrono::{FixedOffset, TimeZone, Utc};

        let chrono = Utc.ymd(2039, 7, 8).and_hms_nano(9, 3, 11, 123_000_000);
        let expected = DateTime::from_str("2039-07-08T09:03:11.123Z", Format::DateTime).unwrap();
        assert_eq!(expected, DateTime::from_chrono_utc(chrono));

        let chrono = Utc.ymd(1000, 7, 8).and_hms_nano(9, 3, 11, 456_000_000);
        let expected = DateTime::from_str("1000-07-08T09:03:11.456Z", Format::DateTime).unwrap();
        assert_eq!(expected, DateTime::from_chrono_utc(chrono));

        let chrono =
            FixedOffset::west(2 * 3600)
                .ymd(2039, 7, 8)
                .and_hms_nano(9, 3, 11, 123_000_000);
        let expected = DateTime::from_str("2039-07-08T11:03:11.123Z", Format::DateTime).unwrap();
        assert_eq!(expected, DateTime::from_chrono_fixed(chrono));
    }

    #[test]
    #[cfg(feature = "convert-chrono")]
    fn to_chrono() {
        use chrono::{TimeZone, Utc};

        let date_time = DateTime::from_str("2039-07-08T09:03:11.123Z", Format::DateTime).unwrap();
        let expected = Utc.ymd(2039, 7, 8).and_hms_nano(9, 3, 11, 123_000_000);
        assert_eq!(expected, date_time.to_chrono_utc());

        let date_time = DateTime::from_str("1000-07-08T09:03:11.456Z", Format::DateTime).unwrap();
        let expected = Utc.ymd(1000, 7, 8).and_hms_nano(9, 3, 11, 456_000_000);
        assert_eq!(expected, date_time.to_chrono_utc());
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
        assert!(matches!(date_time.to_time(), Err(Error::OutOfRange(_))));
        let date_time = DateTime::from_secs_and_nanos(i64::MIN, 0);
        assert!(matches!(date_time.to_time(), Err(Error::OutOfRange(_))));
    }
}
