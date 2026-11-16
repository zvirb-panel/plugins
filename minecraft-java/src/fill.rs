//! Paper's download API, Fill v3, as the Paper engine reads it.

use serde::Deserialize;
use serde::de::DeserializeOwned;
use zvirb_pdk::Error;
use zvirb_pdk::host::http::{self, Method, Request};

const PROJECT: &str = "https://fill.papermc.io/v3/projects/paper";

/// Fill refuses a generic client, and asks for the software and a way to
/// reach its makers (D-0049).
const USER_AGENT: &str = concat!(
    "zvirb-minecraft-java/",
    env!("CARGO_PKG_VERSION"),
    " (https://zvirb.dev)"
);

/// A Minecraft version as Paper builds it.
#[derive(Debug, Deserialize)]
pub struct Version {
    pub version: VersionFacts,
}

#[derive(Debug, Deserialize)]
pub struct VersionFacts {
    pub java: Java,
}

#[derive(Debug, Deserialize)]
pub struct Java {
    pub version: JavaVersion,
}

#[derive(Debug, Deserialize)]
pub struct JavaVersion {
    pub minimum: u32,
}

/// One Paper build of a version.
#[derive(Debug, Deserialize)]
pub struct Build {
    pub downloads: Downloads,
}

#[derive(Debug, Deserialize)]
pub struct Downloads {
    #[serde(rename = "server:default")]
    pub server: Download,
}

#[derive(Debug, Deserialize)]
pub struct Download {
    pub url: String,
    pub checksums: Checksums,
}

#[derive(Debug, Deserialize)]
pub struct Checksums {
    pub sha256: String,
}

/// What Paper knows of Minecraft `version`.
pub fn version(version: &str) -> Result<Version, Error> {
    get(&format!("{PROJECT}/versions/{version}"), version)
}

/// Paper's newest stable build of Minecraft `version`.
pub fn newest_stable_build(version: &str) -> Result<Build, Error> {
    // Fill lists builds newest first.
    let builds: Vec<Build> = get(
        &format!("{PROJECT}/versions/{version}/builds?channel=STABLE"),
        version,
    )?;
    builds
        .into_iter()
        .next()
        .ok_or_else(|| Error::msg(format!("Paper has no stable build of {version} yet")))
}

fn get<T: DeserializeOwned>(url: &str, version: &str) -> Result<T, Error> {
    let mut request = Request::new(Method::Get, url);
    request
        .headers
        .insert("User-Agent".into(), USER_AGENT.into());
    let response = http::send(&request)?;
    match response.status {
        200 => Ok(response.json()?),
        404 => Err(Error::msg(format!("Paper has no version {version}"))),
        status => Err(Error::msg(format!(
            "Paper's download API answered {status}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_java_minimum_of_a_version() {
        let version: Version = serde_json::from_str(include_str!("../testdata/fill-version.json"))
            .expect("the fixture is a version");
        assert_eq!(version.version.java.version.minimum, 21);
    }

    #[test]
    fn reads_the_server_download_of_a_build() {
        let builds: Vec<Build> =
            serde_json::from_str(include_str!("../testdata/fill-stable-builds.json"))
                .expect("the fixture is a list of builds");
        let server = &builds[0].downloads.server;
        assert_eq!(
            server.checksums.sha256,
            "5ee4f542f628a14c644410b08c94ea42e772ef4d29fe92973636b6813d4eaffc"
        );
        assert!(server.url.starts_with("https://fill-data.papermc.io/"));
    }
}
