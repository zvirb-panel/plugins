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
    use zvirb_pdk::view::{
        Paragraph, Permission, Section, SettingsForm, View, ViewPlacement, Words, error,
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

    // The API reads a view file without running the module, so the file is
    // what ships; building the same view with the PDK's types proves the file
    // is one the view schema takes.
    #[test]
    fn each_settings_view_file_is_the_view_the_builders_make() -> Result<(), error::ConversionError>
    {
        for (engine, file) in [
            ("paper", include_str!("../views/paper-settings.json")),
            ("vanilla", include_str!("../views/vanilla-settings.json")),
        ] {
            // A tab sits under the server's own head, so it names itself with a
            // section, not a second page title.
            let note = Paragraph::try_from(Paragraph::builder().text(Words::try_from(
                "Changes apply the next time the server starts.",
            )?))?;
            let settings_form = SettingsForm::try_from(SettingsForm::builder())?;
            let section = Section::try_from(
                Section::builder()
                    .heading(Words::try_from("Settings")?)
                    .body(vec![note.into(), settings_form.into()]),
            )?;
            let view = View::try_from(
                View::builder()
                    .id(format!("minecraft-java.{engine}-settings"))
                    .placement(ViewPlacement::InstanceTab)
                    .title("Settings")
                    .order(10_u64)
                    .permission(Permission::from("instances.manage".to_owned()))
                    .body(vec![section.into()]),
            )?;

            let file: Value = serde_json::from_str(file).expect("the view file is JSON");
            assert_eq!(
                serde_json::to_value(view).expect("a view serializes"),
                file,
                "{engine}"
            );
        }
        Ok(())
    }
}
