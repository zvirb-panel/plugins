//! The Vanilla engine: Minecraft servers on Mojang's own server jar.

use serde_json::{Map, Value};
use zvirb_pdk::Error;
use zvirb_pdk::capability::{Capability, VersionCatalog};
use zvirb_pdk::engine::{Artifact, ChecksumAlgorithm, Engine, Limits, Plan, Refusal, Unpack, User};

use crate::{eula, java, mojang};

pub struct Vanilla;

/// Where the plan puts the server jar, in the instance's folder.
const JAR: &str = "server.jar";

impl Engine for Vanilla {
    const ID: &'static str = "vanilla";
    const CAPABILITIES: &'static [Capability] = &[Capability::version_catalog::<Self>()];

    fn plan(settings: &Map<String, Value>) -> Result<Plan, Error> {
        let (accepted, version) = accepted_version(settings)?;
        plan_for(accepted, version, &mojang::version(version)?)
    }
}

impl VersionCatalog for Vanilla {
    fn versions() -> Result<Vec<String>, Error> {
        mojang::releases()
    }
}

/// The Minecraft version settings ask for, once they accept the EULA, so a
/// server without it never reaches Mojang's manifest.
fn accepted_version(settings: &Map<String, Value>) -> Result<(eula::Accepted, &str), Error> {
    let accepted = eula::require_accepted(settings)?;
    let version = settings
        .get("version")
        .and_then(Value::as_str)
        .ok_or_else(|| Refusal::new("version", "a Minecraft version is required"))?;
    Ok((accepted, version))
}

/// The plan that runs Mojang's server for `version`, once the EULA is
/// accepted.
fn plan_for(accepted: eula::Accepted, id: &str, version: &mojang::Version) -> Result<Plan, Error> {
    let server =
        version.downloads.server.as_ref().ok_or_else(|| {
            Refusal::new("version", format!("Mojang publishes no server for {id}"))
        })?;
    Ok(Plan {
        image: java::image(version.java.major)?.into(),
        env: std::collections::BTreeMap::new(),
        command: ["java", "-jar", JAR, "nogui"].map(String::from).to_vec(),
        limits: Limits::default(),
        artifacts: vec![Artifact {
            url: server.url.clone(),
            checksum_algorithm: ChecksumAlgorithm::Sha1,
            checksum: server.sha1.clone(),
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
    use serde_json::json;

    use super::*;

    fn accepted() -> eula::Accepted {
        let settings = json!({ "version": "1.21.4", "eula": true });
        let (accepted, _) =
            accepted_version(settings.as_object().expect("an object")).expect("accepted");
        accepted
    }

    fn fixture() -> mojang::Version {
        serde_json::from_str(include_str!("../testdata/mojang-version.json"))
            .expect("the fixture is a version")
    }

    #[test]
    fn plans_mojangs_server_jar_on_the_java_it_lists() {
        let plan = plan_for(accepted(), "1.21.4", &fixture()).expect("the fixture plans");

        assert!(
            plan.image.starts_with("eclipse-temurin:21-jre@sha256:"),
            "{}",
            plan.image
        );
        assert_eq!(plan.command, ["java", "-jar", "server.jar", "nogui"]);
        let [jar] = plan.artifacts.as_slice() else {
            panic!("want one artifact, got {:?}", plan.artifacts);
        };
        assert_eq!(
            jar.url,
            "https://piston-data.mojang.com/v1/objects/4707d00eb834b446575d89a61a11b5d548d8c001/server.jar"
        );
        assert_eq!(jar.checksum_algorithm, ChecksumAlgorithm::Sha1);
        assert_eq!(jar.checksum, "4707d00eb834b446575d89a61a11b5d548d8c001");
        let [eula] = plan.seeded_files.as_slice() else {
            panic!("want one seeded file, got {:?}", plan.seeded_files);
        };
        assert_eq!(eula.content, b"eula=true\n");
    }

    #[test]
    fn a_version_without_a_server_is_refused() {
        let mut version = fixture();
        version.downloads.server = None;
        let err = plan_for(accepted(), "rd-132211", &version).expect_err("no server, no plan");
        let refusal = err.downcast_ref::<Refusal>().expect("a refusal");
        assert_eq!(refusal.setting, "version");
    }
}
