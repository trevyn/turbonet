#![forbid(unsafe_code)]
#![allow(non_camel_case_types)]
#![doc = include_str!("../README.md")]

use turbosql::Turbosql;

#[derive(Turbosql)]
struct _Turbonet_Peers {
 rowid: Option<i64>,
 ip: Option<u32>,
 last_seen_ms: Option<i64>,
}
