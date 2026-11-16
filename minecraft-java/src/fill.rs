//! Paper's download API, Fill v3, as the Paper engine reads it.

use serde::Deserialize;
use serde::de::DeserializeOwned;
use zvirb_pdk::Error;
use zvirb_pdk::engine::Refusal;
use zvirb_pdk::host::http::{self, Method, Request, Response};

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
    let builds = get(
        &format!("{PROJECT}/versions/{version}/builds?channel=STABLE"),
        version,
    )?;
    newest(builds, version)
}

fn newest(builds: Vec<Build>, version: &str) -> Result<Build, Error> {
    builds.into_iter().next().ok_or_else(|| {
        Refusal::new(
            "version",
            format!("Paper has no stable build of {version} yet"),
        )
        .into()
    })
}

fn get<T: DeserializeOwned>(url: &str, version: &str) -> Result<T, Error> {
    let mut request = Request::new(Method::Get, url);
    request
        .headers
        .insert("User-Agent".into(), USER_AGENT.into());
    read(&http::send(&request)?, url, version)
}

/// What Fill's `response` to `url` holds. Fill answers 404 for a version it
/// doesn't know, which the person can fix; anything else is Fill's fault.
fn read<T: DeserializeOwned>(response: &Response, url: &str, version: &str) -> Result<T, Error> {
    match response.status {
        200 => response
            .json()
            .map_err(|err| Error::msg(format!("read Paper's answer from {url}: {err}"))),
        404 => Err(Refusal::new("version", format!("Paper has no version {version}")).into()),
        status => Err(Error::msg(format!(
            "Paper's download API answered {status}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    fn answer(status: u16) -> Response {
        Response {
            status,
            headers: BTreeMap::new(),
            body: Vec::new(),
        }
    }

    fn refused_setting(err: &Error) -> Option<&str> {
        err.downcast_ref::<Refusal>().map(|r| r.setting.as_str())
    }

    #[test]
    fn a_version_fill_doesnt_know_is_refused() {
        let err = read::<Version>(&answer(404), PROJECT, "9.9.9").expect_err("404 is no version");
        assert_eq!(refused_setting(&err), Some("version"));
        assert!(
            err.to_string().contains("Paper has no version 9.9.9"),
            "{err}"
        );
    }

    #[test]
    fn a_version_without_a_stable_build_is_refused() {
        let err = newest(Vec::new(), "26.4").expect_err("no builds, no build");
        assert_eq!(refused_setting(&err), Some("version"));
    }

    #[test]
    fn fill_failing_is_not_the_persons_to_fix() {
        let err = read::<Version>(&answer(503), PROJECT, "1.21.4").expect_err("503 fails");
        assert_eq!(refused_setting(&err), None);
    }
}
