//! The `server.properties` keys the settings edit. Each is a setting of the
//! same name, and the plan patches the file with the ones an instance sets,
//! so every key the settings don't name stays as the server wrote it.

use std::fmt::Write as _;

use serde_json::{Map, Value};
use zvirb_pdk::Error;
use zvirb_pdk::engine::{Refusal, SeedKind, SeededFile};

/// The keys, in the order the settings schemas list them.
pub const KEYS: &[&str] = &[
    "motd",
    "max-players",
    "difficulty",
    "gamemode",
    "hardcore",
    "pvp",
    "online-mode",
    "white-list",
    "view-distance",
    "simulation-distance",
    "spawn-protection",
    "level-seed",
    "allow-flight",
    "enable-command-block",
    "allow-nether",
    "spawn-monsters",
    "level-type",
    "force-gamemode",
];

/// The patch of `server.properties` for the keys `settings` sets, or none
/// when it sets none, so the server writes its own defaults.
pub fn patch(settings: &Map<String, Value>) -> Result<Option<SeededFile>, Error> {
    let mut content = String::new();
    for key in KEYS {
        let Some(value) = settings.get(*key) else {
            continue;
        };
        content.push_str(key);
        content.push('=');
        content.push_str(&text_of(key, value)?);
        content.push('\n');
    }
    if content.is_empty() {
        return Ok(None);
    }
    Ok(Some(SeededFile {
        path: "server.properties".into(),
        content: content.into_bytes(),
        mode: None,
        kind: SeedKind::Properties,
    }))
}

/// A setting's value as a properties value. The schema holds each to its
/// type; this refuses one stored before the schema asked.
fn text_of(key: &str, value: &Value) -> Result<String, Error> {
    match value {
        Value::String(text) => Ok(escaped(text)),
        Value::Bool(flag) => Ok(flag.to_string()),
        Value::Number(number) if number.is_i64() => Ok(number.to_string()),
        _ => Err(Refusal::new(key, "give the value the setting describes").into()),
    }
}

/// `text` as a properties file reads it whatever its encoding: a backslash
/// doubled, and every character outside ASCII as a `\u` escape.
fn escaped(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            ' '..='~' => out.push(c),
            _ => {
                for unit in c.encode_utf16(&mut [0; 2]) {
                    // Writing to a String can't fail.
                    let _ = write!(out, "\\u{unit:04x}");
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn settings(value: &Value) -> Map<String, Value> {
        value.as_object().expect("settings are an object").clone()
    }

    #[test]
    fn each_setting_lands_under_its_property_name() {
        let file = patch(&settings(&json!({
            "version": "1.21.4",
            "eula": true,
            "motd": "Survival",
            "max-players": 50,
            "pvp": false,
            "level-type": "minecraft:flat",
            "level-seed": "",
        })))
        .expect("a patch")
        .expect("some keys");
        assert_eq!(file.path, "server.properties");
        assert_eq!(file.kind, SeedKind::Properties);
        assert_eq!(
            String::from_utf8(file.content).unwrap(),
            "motd=Survival\nmax-players=50\npvp=false\nlevel-seed=\nlevel-type=minecraft:flat\n"
        );
    }

    #[test]
    fn both_schemas_name_every_key_and_no_other() {
        for schema in [
            include_str!("../settings/paper.schema.json"),
            include_str!("../settings/vanilla.schema.json"),
        ] {
            let schema: Value = serde_json::from_str(schema).expect("the schema is JSON");
            let mut named: Vec<&str> = schema["properties"]
                .as_object()
                .expect("the schema has properties")
                .keys()
                .map(String::as_str)
                .filter(|name| !["version", "eula"].contains(name))
                .collect();
            let mut keys = KEYS.to_vec();
            named.sort_unstable();
            keys.sort_unstable();
            assert_eq!(named, keys);
        }
    }

    #[test]
    fn no_property_set_is_no_patch() {
        let file =
            patch(&settings(&json!({ "version": "1.21.4", "eula": true }))).expect("a patch");
        assert_eq!(file, None);
    }

    #[test]
    fn text_outside_ascii_and_backslashes_are_escaped() {
        let file = patch(&settings(&json!({ "motd": "Čau \\o/ 🎮" })))
            .expect("a patch")
            .expect("a key");
        assert_eq!(
            String::from_utf8(file.content).unwrap(),
            "motd=\\u010cau \\\\o/ \\ud83c\\udfae\n"
        );
    }

    #[test]
    fn a_value_of_the_wrong_kind_is_refused() {
        let Err(err) = patch(&settings(&json!({ "max-players": [50] }))) else {
            panic!("a list is taken as max-players");
        };
        let refusal = err.downcast_ref::<Refusal>().expect("a refusal");
        assert_eq!(refusal.setting, "max-players");
    }
}
