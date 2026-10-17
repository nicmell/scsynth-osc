//! The JSON form of the messages, with the feature `serde`.
#![cfg(feature = "serde")]

use scsynth_osc::command::top_level::{DumpMode, Notify, SyncId};
use scsynth_osc::reply::top_level::StatusReply;
use scsynth_osc::{Command, Reply};
use serde_json::json;

fn round_trip<T>(value: &T, json: serde_json::Value)
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
{
    assert_eq!(serde_json::to_value(value).expect("serializes"), json);
    let back: T = serde_json::from_value(json).expect("deserializes");
    assert_eq!(&back, value);
}

#[test]
fn a_command_is_its_address_and_its_args() {
    round_trip(&Command::Status, json!({"address": "/status"}));
    round_trip(
        &Command::Sync(SyncId(7)),
        json!({"address": "/sync", "args": 7}),
    );
    round_trip(
        &Command::DumpOsc(DumpMode::Parsed),
        json!({"address": "/dumpOSC", "args": "parsed"}),
    );
    let notify = Notify {
        on: true,
        client_id: None,
    };
    round_trip(
        &Command::Notify(notify),
        json!({"address": "/notify", "args": {"on": true, "client_id": null}}),
    );
}

#[test]
fn a_reply_is_its_address_and_its_fields() {
    let status = StatusReply {
        ugens: 1,
        synths: 2,
        groups: 3,
        synthdefs: 4,
        avg_cpu: 0.5,
        peak_cpu: 1.5,
        nominal_sample_rate: 48_000.0,
        actual_sample_rate: 47_999.5,
    };
    round_trip(
        &Reply::StatusReply(status),
        json!({"address": "/status.reply", "args": {
            "ugens": 1, "synths": 2, "groups": 3, "synthdefs": 4,
            "avg_cpu": 0.5, "peak_cpu": 1.5,
            "nominal_sample_rate": 48000.0, "actual_sample_rate": 47999.5
        }}),
    );
}
