//! Parse/display opaque scalars round-trip (FTS vendor addition).
//!
//! facet models `uuid::Uuid`, the `chrono` types, `url::Url`, … as
//! `Def::Scalar` shapes whose only representation is the `display` /
//! `parse` pair in their vtable. Upstream phon (rc.5 through rc.7) has no
//! derive branch for them, so a codec for any type containing one fails to
//! lower — and vox's opaque encode thunk is `extern "C"`, so inside an RPC
//! reply that failure aborts the process.

use facet::Facet;
use phon::api::Codec;

#[derive(Debug, Clone, PartialEq, Facet)]
struct Row {
    id: uuid::Uuid,
    at: chrono::DateTime<chrono::Utc>,
    maybe: Option<uuid::Uuid>,
    many: Vec<uuid::Uuid>,
    label: String,
}

fn roundtrip<T>(value: T)
where
    T: for<'facet> Facet<'facet> + PartialEq + std::fmt::Debug,
{
    let codec = Codec::<T>::new().expect("a display/parse scalar should lower");
    let bytes = codec.encode(&value).expect("should encode");
    let back = codec.decode(&bytes).expect("should decode");
    assert_eq!(back, value);
}

#[test]
fn a_bare_uuid_round_trips() {
    roundtrip(uuid::Uuid::from_u128(0x0123_4567_89ab_cdef_0123_4567_89ab_cdef));
}

#[test]
fn scalars_inside_a_struct_round_trip() {
    let id = uuid::Uuid::from_u128(42);
    roundtrip(Row {
        id,
        at: chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap(),
        maybe: Some(id),
        many: vec![id, uuid::Uuid::nil()],
        label: "row".into(),
    });
}
