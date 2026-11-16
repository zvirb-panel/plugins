//! A plugin with one engine, to prove a crate outside `zvirb` builds against the PDK.

use serde_json::{Map, Value};
use zvirb_pdk::Error;
use zvirb_pdk::engine::{Engine, Limits, Plan, PortNeed, Protocol, Stop, User};

struct Sample;

impl Engine for Sample {
    const ID: &'static str = "sample";

    fn ports() -> Vec<PortNeed> {
        vec![PortNeed {
            name: "game".into(),
            container_port: 25565,
            protocols: vec![Protocol::Tcp],
            preferred: 0,
        }]
    }

    fn plan(_settings: &Map<String, Value>) -> Result<Plan, Error> {
        Ok(Plan {
            image: "zvirb/sample:latest".into(),
            env: std::collections::BTreeMap::new(),
            command: Vec::new(),
            limits: Limits::default(),
            artifacts: Vec::new(),
            seeded_files: Vec::new(),
            stop: Stop {
                command: "stop".into(),
                signal: None,
                grace_seconds: 10,
            },
            user: User {
                uid: 1000,
                gid: 1000,
            },
        })
    }
}

zvirb_pdk::plugin! { engines: [Sample] }

#[cfg(test)]
mod tests {
    use std::process::Command;

    use serde_json::Value;

    // `mise run check` installs `zvirb-pdk` at the PDK tag the crate builds against,
    // so the schema is the one that PDK publishes.
    fn manifest_schema() -> Value {
        let output = Command::new("zvirb-pdk")
            .arg("schema")
            .output()
            .expect("zvirb-pdk is not on PATH; run `mise run check`");
        assert!(output.status.success(), "zvirb-pdk schema failed");
        serde_json::from_slice(&output.stdout).expect("zvirb-pdk schema prints JSON")
    }

    #[test]
    fn manifest_matches_the_pdk_schema() {
        let manifest: Value = serde_json::from_str(include_str!("../zvirb-plugin.json"))
            .expect("zvirb-plugin.json is JSON");
        let validator = jsonschema::validator_for(&manifest_schema()).expect("the schema compiles");
        let problems: Vec<String> = validator
            .iter_errors(&manifest)
            .map(|error| format!("{}: {error}", error.instance_path()))
            .collect();
        assert!(problems.is_empty(), "{}", problems.join("\n"));
    }
}
