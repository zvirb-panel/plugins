//! The Paper engine: Minecraft servers on Paper's builds.

use serde_json::{Map, Value};
use zvirb_pdk::Error;
use zvirb_pdk::capability::{Capability, VersionCatalog};
use zvirb_pdk::engine::{Artifact, ChecksumAlgorithm, Engine, Limits, Plan, Refusal, Unpack, User};

use crate::{eula, fill, java};

pub struct Paper;

/// Where the plan puts the server jar, in the instance's folder.
const JAR: &str = "server.jar";

impl Engine for Paper {
    const ID: &'static str = "paper";
    const CAPABILITIES: &'static [Capability] = &[Capability::version_catalog::<Self>()];

    fn plan(settings: &Map<String, Value>) -> Result<Plan, Error> {
        let (accepted, version) = accepted_version(settings)?;
        plan_for(
            accepted,
            &fill::version(version)?,
            &fill::newest_stable_build(version)?,
        )
    }
}

impl VersionCatalog for Paper {
    fn versions() -> Result<Vec<String>, Error> {
        fill::releases()
    }
}

/// The Minecraft version settings ask for, once they accept the EULA, so a
/// server without it never reaches Paper's download API.
fn accepted_version(settings: &Map<String, Value>) -> Result<(eula::Accepted, &str), Error> {
    let accepted = eula::require_accepted(settings)?;
    let version = settings
        .get("version")
        .and_then(Value::as_str)
        .ok_or_else(|| Refusal::new("version", "a Minecraft version is required"))?;
    Ok((accepted, version))
}

/// The plan that runs `build` of a version, once the EULA is accepted.
fn plan_for(
    accepted: eula::Accepted,
    version: &fill::Version,
    build: &fill::Build,
) -> Result<Plan, Error> {
    let server = &build.downloads.server;
    Ok(Plan {
        image: java::image(version.version.java.version.minimum)?.into(),
        env: std::collections::BTreeMap::new(),
        command: ["java", "-jar", JAR, "--nogui"].map(String::from).to_vec(),
        limits: Limits::default(),
        artifacts: vec![Artifact {
            url: server.url.clone(),
            checksum_algorithm: ChecksumAlgorithm::Sha256,
            checksum: server.checksums.sha256.clone(),
            destination: JAR.into(),
            unpack: Unpack::None,
            strip_components: 0,
        }],
        seeded_files: vec![eula::accepted_file(accepted)],
        // The `eclipse-temurin` images are Ubuntu, whose first user is 1000.
        user: User {
            uid: 1000,
            gid: 1000,
        },
    })
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;

    fn fixtures() -> (fill::Version, fill::Build) {
        let version = serde_json::from_str(include_str!("../testdata/fill-version.json"))
            .expect("the fixture is a version");
        let builds: Vec<fill::Build> =
            serde_json::from_str(include_str!("../testdata/fill-stable-builds.json"))
                .expect("the fixture is a list of builds");
        (version, builds.into_iter().next().expect("one build"))
    }

    #[test]
    fn plans_the_build_on_its_java_with_the_eula_accepted() {
        let (version, build) = fixtures();
        let settings = json!({ "version": "1.21.4", "eula": true });
        let (accepted, _) =
            accepted_version(settings.as_object().expect("an object")).expect("accepted");
        let plan = plan_for(accepted, &version, &build).expect("the fixtures plan");

        assert!(
            plan.image.starts_with("eclipse-temurin:21-jre@sha256:"),
            "{}",
            plan.image
        );
        assert_eq!(plan.command, ["java", "-jar", "server.jar", "--nogui"]);
        let [jar] = plan.artifacts.as_slice() else {
            panic!("want one artifact, got {:?}", plan.artifacts);
        };
        assert_eq!(
            jar.url,
            "https://fill-data.papermc.io/v1/objects/5ee4f542f628a14c644410b08c94ea42e772ef4d29fe92973636b6813d4eaffc/paper-1.21.4-232.jar"
        );
        assert_eq!(jar.checksum_algorithm, ChecksumAlgorithm::Sha256);
        assert_eq!(
            jar.checksum,
            "5ee4f542f628a14c644410b08c94ea42e772ef4d29fe92973636b6813d4eaffc"
        );
        assert_eq!(jar.destination, "server.jar");
        let [eula] = plan.seeded_files.as_slice() else {
            panic!("want one seeded file, got {:?}", plan.seeded_files);
        };
        assert_eq!(eula.path, "eula.txt");
        assert_eq!(eula.content, b"eula=true\n");
    }

    fn settings_problems(settings: &Value) -> Vec<String> {
        let schema: Value = serde_json::from_str(include_str!("../settings/paper.schema.json"))
            .expect("the settings schema is JSON");
        let validator = jsonschema::validator_for(&schema).expect("the settings schema compiles");
        validator
            .iter_errors(settings)
            .map(|error| format!("{}: {error}", error.instance_path()))
            .collect()
    }

    // The API checks settings against this schema before it calls the plan.
    #[test]
    fn the_settings_schema_refuses_a_create_without_the_eula() {
        assert_eq!(
            settings_problems(&json!({ "version": "1.21.4", "eula": true })),
            Vec::<String>::new()
        );
        for refused in [
            json!({ "version": "1.21.4" }),
            json!({ "version": "1.21.4", "eula": false }),
        ] {
            assert!(!settings_problems(&refused).is_empty(), "{refused}");
        }
    }

    #[test]
    fn the_settings_schema_refuses_a_version_that_would_leave_the_url_path() {
        for refused in ["../1.21.4", "1.21.4?channel=ALPHA", "1.21.4/builds", ""] {
            let settings = json!({ "version": refused, "eula": true });
            assert!(!settings_problems(&settings).is_empty(), "{refused}");
        }
    }
}
