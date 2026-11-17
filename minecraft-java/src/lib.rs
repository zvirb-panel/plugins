//! The Minecraft: Java plugin: servers for Minecraft's Java edition.

mod eula;
mod fill;
mod java;
mod mojang;
mod paper;
mod properties;
mod vanilla;

zvirb_pdk::plugin! { engines: [paper::Paper, vanilla::Vanilla] }

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
