//! Paper's download API, Fill v3, as the Paper engine reads it.

use serde::Deserialize;
use serde::de::DeserializeOwned;
use zvirb_pdk::Error;
use zvirb_pdk::engine::Refusal;
use zvirb_pdk::host;
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
    pub id: String,
    pub java: Java,
}

/// Every version Paper builds, newest first.
#[derive(Debug, Deserialize)]
struct Versions {
    versions: Vec<Version>,
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
    get(&format!("{PROJECT}/versions/{version}"))?.ok_or_else(|| no_version(version))
}

/// Paper's newest stable build of Minecraft `version`.
pub fn newest_stable_build(version: &str) -> Result<Build, Error> {
    // Fill lists builds newest first.
    let builds = get(&format!(
        "{PROJECT}/versions/{version}/builds?channel=STABLE"
    ))?
    .ok_or_else(|| no_version(version))?;
    newest(builds, version)
}

/// The Minecraft releases Paper builds, newest first.
pub fn releases() -> Result<Vec<String>, Error> {
    let url = format!("{PROJECT}/versions");
    let listed = get(&url)?
        .ok_or_else(|| Error::msg(format!("Paper's download API has no version list at {url}")))?;
    Ok(release_ids(listed))
}

// Fill lists pre-releases and release candidates beside releases, and only
// their ids set them apart: 1.21.11-pre5, 26.3-rc-3.
fn release_ids(listed: Versions) -> Vec<String> {
    listed
        .versions
        .into_iter()
        .map(|listed| listed.version.id)
        .filter(|id| !id.contains('-'))
        .collect()
}

fn no_version(version: &str) -> Error {
    Refusal::new("version", format!("Paper has no version {version}")).into()
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

fn get<T: DeserializeOwned>(url: &str) -> Result<Option<T>, Error> {
    let mut request = Request::new(Method::Get, url);
    request
        .headers
        .insert("User-Agent".into(), USER_AGENT.into());
    read(http::send(&request), url)
}

/// What Fill's answer to `url` holds, or `None` when Fill doesn't know what
/// the URL names. Each failure names Paper's download API, so a catalog that
/// can't be read never passes for an empty one.
fn read<T: DeserializeOwned>(
    sent: Result<Response, host::Error>,
    url: &str,
) -> Result<Option<T>, Error> {
    let response =
        sent.map_err(|err| Error::msg(format!("ask Paper's download API for {url}: {err}")))?;
    match response.status {
        200 => response
            .json()
            .map(Some)
            .map_err(|err| Error::msg(format!("read Paper's answer from {url}: {err}"))),
        404 => Ok(None),
        status => Err(Error::msg(format!(
            "Paper's download API answered {status} for {url}"
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
        assert!(matches!(
            read::<Version>(Ok(answer(404)), PROJECT),
            Ok(None)
        ));
        let err = no_version("9.9.9");
        assert_eq!(refused_setting(&err), Some("version"));
        assert_eq!(err.to_string(), "version: Paper has no version 9.9.9");
    }

    #[test]
    fn a_version_without_a_stable_build_is_refused() {
        let err = newest(Vec::new(), "26.4").expect_err("no builds, no build");
        assert_eq!(refused_setting(&err), Some("version"));
    }

    #[test]
    fn the_releases_are_newest_first_without_pre_releases() {
        let listed: Versions = serde_json::from_str(include_str!("../testdata/fill-versions.json"))
            .expect("the fixture is a version list");
        assert_eq!(
            release_ids(listed),
            ["26.3", "26.2", "1.21.11", "1.21.4", "1.7.10"]
        );
    }

    #[test]
    fn fill_failing_or_timing_out_names_it_and_isnt_the_persons_to_fix() {
        let timed_out = Err(host::Error::Refused {
            kind: "timed_out".into(),
            message: "the request passed its deadline".into(),
        });
        for sent in [Ok(answer(503)), timed_out] {
            let err = read::<Versions>(sent, PROJECT).expect_err("no catalog");
            assert_eq!(refused_setting(&err), None);
            assert!(err.to_string().contains("Paper's download API"), "{err}");
        }
    }
}
