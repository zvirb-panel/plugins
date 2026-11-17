//! The Minecraft EULA, which a server runs only once its owner accepts it.

use serde_json::{Map, Value};
use zvirb_pdk::Error;
use zvirb_pdk::engine::{Refusal, SeedKind, SeededFile};

/// The settings key the settings schemas ask for the EULA under.
const SETTING: &str = "eula";

/// That an instance's settings accept the EULA. Only [`require_accepted`]
/// makes one, so `eula.txt` can't be seeded without the check.
pub struct Accepted(());

/// Refuses settings that don't carry the owner's acceptance. The API checks
/// the settings schema first, so this holds the line only for an instance
/// whose settings were stored before the schema asked.
pub fn require_accepted(settings: &Map<String, Value>) -> Result<Accepted, Error> {
    if settings.get(SETTING) == Some(&Value::Bool(true)) {
        Ok(Accepted(()))
    } else {
        Err(Refusal::new(
            SETTING,
            "the Minecraft EULA has to be accepted before the server is created",
        )
        .into())
    }
}

/// The `eula.txt` the server reads its acceptance from.
pub fn accepted_file(_: Accepted) -> SeededFile {
    SeededFile {
        path: "eula.txt".into(),
        content: b"eula=true\n".to_vec(),
        mode: None,
        kind: SeedKind::Whole,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn settings(value: &Value) -> Map<String, Value> {
        value.as_object().expect("settings are an object").clone()
    }

    #[test]
    fn only_true_is_acceptance() {
        assert!(require_accepted(&settings(&json!({ "eula": true }))).is_ok());
        for refused in [
            json!({}),
            json!({ "eula": false }),
            json!({ "eula": "true" }),
        ] {
            let Err(err) = require_accepted(&settings(&refused)) else {
                panic!("{refused} is accepted");
            };
            let refusal = err.downcast_ref::<Refusal>().expect("a refusal");
            assert_eq!(refusal.setting, "eula", "{refused}");
        }
    }
}
