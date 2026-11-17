//! A plugin with one engine and one view, to prove a crate outside `zvirb`
//! builds against the PDK.

use serde_json::{Map, Value};
use zvirb_pdk::Error;
use zvirb_pdk::engine::{Engine, Limits, Plan, User};

struct Sample;

impl Engine for Sample {
    const ID: &'static str = "sample";

    fn plan(_settings: &Map<String, Value>) -> Result<Plan, Error> {
        Ok(Plan {
            image: "zvirb/sample:latest".into(),
            env: std::collections::BTreeMap::new(),
            command: Vec::new(),
            limits: Limits::default(),
            artifacts: Vec::new(),
            seeded_files: Vec::new(),
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
    use zvirb_pdk::view::{
        KeyValues, KeyValuesItemsItem, Page, Permission, View, ViewPlacement, error,
    };

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

    fn row(label: &str, value: &str) -> Result<KeyValuesItemsItem, error::ConversionError> {
        KeyValuesItemsItem::try_from(KeyValuesItemsItem::builder().label(label).value(value))
    }

    // The API reads the view file without running the module, so the file is
    // what ships; building the same view with the PDK's types proves the file
    // is one the view schema takes.
    #[test]
    fn the_view_file_is_the_view_the_builders_make() -> Result<(), error::ConversionError> {
        let page = Page::try_from(Page::builder().title("About"))?;
        let facts = KeyValues::try_from(KeyValues::builder().items(vec![
            row("Engine", "Sample")?,
            row("Image", "zvirb/sample:latest")?,
        ]))?;
        let view = View::try_from(
            View::builder()
                .id("sample.about".to_owned())
                .placement(ViewPlacement::InstanceTab)
                .title("About")
                .order(0_u64)
                .permission(Permission::from("instances.manage".to_owned()))
                .body(vec![page.into(), facts.into()]),
        )?;

        let file: Value = serde_json::from_str(include_str!("../views/about.json"))
            .expect("views/about.json is JSON");
        assert_eq!(serde_json::to_value(view).expect("a view serializes"), file);
        Ok(())
    }
}
