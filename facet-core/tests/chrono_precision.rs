//! Timestamps keep the precision they were created with.
//!
//! `chrono`'s types are opaque scalars here: their only encode path is
//! the `display` fn in each vtable, so whatever that fn writes is what
//! *every* facet format writes — JSON, TOML, and any binary format built
//! on the same reflection.
//!
//! Upstream 0.50.0-rc.5 formatted the `DateTime` types with
//! `SecondsFormat::Secs` and the naive types with `%H:%M:%S`, both of
//! which drop the fraction. That is a silent, one-directional loss: the
//! value round-trips without error and comes back rounded, so anything
//! ordered by a timestamp can invert when two events share a second.
//!
//! The fix is `SecondsFormat::AutoSi` and `%.f`, both of which write
//! *nothing* when the fraction is zero — so a whole-second value formats
//! exactly as it did before, and only the previously-lossy case changes.

#![cfg(feature = "chrono")]

use chrono::{DateTime, FixedOffset, NaiveDateTime, NaiveTime, TimeZone as _, Utc};
use facet_core::Facet;

/// What a format writes for a value.
///
/// The `display` vtable entry, reached through `Shape::call_display` —
/// the only encode path an opaque scalar has, and therefore the one
/// every format goes through.
fn displayed<'a, T: Facet<'a>>(value: &T) -> String {
    struct Displayer<'v, T>(&'v T);

    impl<'a, T: Facet<'a>> core::fmt::Display for Displayer<'_, T> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            let ptr = facet_core::PtrConst::new(core::ptr::from_ref(self.0).cast::<u8>());
            // SAFETY: `ptr` points at a live `T`, and the shape is `T`'s.
            unsafe { T::SHAPE.call_display(ptr, f) }
                .expect("chrono types carry a display fn")
        }
    }

    Displayer(value).to_string()
}

#[test]
fn datetime_utc_keeps_its_fraction() {
    let dt: DateTime<Utc> = Utc.timestamp_nanos(1_755_400_254_123_456_789);
    assert_eq!(displayed(&dt), "2025-08-17T03:10:54.123456789Z");
}

/// The half that keeps existing data byte-identical.
#[test]
fn a_whole_second_is_written_exactly_as_before() {
    let dt: DateTime<Utc> = Utc.timestamp_nanos(1_755_400_254_000_000_000);
    assert_eq!(displayed(&dt), "2025-08-17T03:10:54Z");
}

#[test]
fn datetime_fixed_offset_keeps_its_fraction() {
    let dt: DateTime<FixedOffset> = DateTime::parse_from_rfc3339("2025-08-17T02:30:54.123456789Z")
        .expect("parse");
    assert!(displayed(&dt).contains(".123456789"), "{}", displayed(&dt));
}

#[test]
fn naive_datetime_keeps_its_fraction() {
    let dt: NaiveDateTime = NaiveDateTime::parse_from_str("2025-08-17T02:30:54.123456789", "%Y-%m-%dT%H:%M:%S%.f")
        .expect("parse");
    assert_eq!(displayed(&dt), "2025-08-17T02:30:54.123456789");
}

#[test]
fn naive_time_keeps_its_fraction() {
    let t: NaiveTime = NaiveTime::parse_from_str("02:30:54.123456789", "%H:%M:%S%.f").expect("parse");
    assert_eq!(displayed(&t), "02:30:54.123456789");
}
