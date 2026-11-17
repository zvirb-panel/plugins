//! Mojang's version manifest, as the Vanilla engine reads it.

use serde::Deserialize;
use serde::de::DeserializeOwned;
use zvirb_pdk::Error;
use zvirb_pdk::engine::Refusal;
use zvirb_pdk::host;
use zvirb_pdk::host::http::{self, Method, Request, Response};

const MANIFEST: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

/// Every version Mojang publishes, newest first.
#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub versions: Vec<Listed>,
}

/// A version as the manifest lists it, pointing at its own file.
#[derive(Debug, Deserialize)]
pub struct Listed {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub url: String,
}

/// A version's own file: what it runs on and what it downloads.
#[derive(Debug, Deserialize)]
pub struct Version {
    #[serde(rename = "javaVersion")]
    pub java: Java,
    pub downloads: Downloads,
}

#[derive(Debug, Deserialize)]
pub struct Java {
    #[serde(rename = "majorVersion")]
    pub major: u32,
}

#[derive(Debug, Deserialize)]
pub struct Downloads {
    pub server: Option<Download>,
}

#[derive(Debug, Deserialize)]
pub struct Download {
    pub url: String,
    pub sha1: String,
}

/// The file of Minecraft `version`, from the manifest's entry for it.
pub fn version(version: &str) -> Result<Version, Error> {
    let manifest: Manifest = get(MANIFEST)?;
    get(&listed(&manifest, version)?.url)
}

/// The Minecraft releases Mojang publishes, newest first.
pub fn releases() -> Result<Vec<String>, Error> {
    Ok(release_ids(&get(MANIFEST)?))
}

// Snapshots and the old alphas and betas sit in the same list.
fn release_ids(manifest: &Manifest) -> Vec<String> {
    manifest
        .versions
        .iter()
        .filter(|listed| listed.kind == "release")
        .map(|listed| listed.id.clone())
        .collect()
}

fn listed<'a>(manifest: &'a Manifest, version: &str) -> Result<&'a Listed, Error> {
    manifest
        .versions
        .iter()
        .find(|listed| listed.id == version)
        .ok_or_else(|| Refusal::new("version", format!("Mojang has no version {version}")).into())
}

fn get<T: DeserializeOwned>(url: &str) -> Result<T, Error> {
    read(http::send(&Request::new(Method::Get, url)), url)
}

/// What Mojang's answer to `url` holds. Each failure names Mojang, so a
/// catalog that can't be read never passes for an empty one.
fn read<T: DeserializeOwned>(sent: Result<Response, host::Error>, url: &str) -> Result<T, Error> {
    let response = sent.map_err(|err| Error::msg(format!("ask Mojang for {url}: {err}")))?;
    match response.status {
        200 => response
            .json()
            .map_err(|err| Error::msg(format!("read Mojang's answer from {url}: {err}"))),
        status => Err(Error::msg(format!("Mojang's {url} answered {status}"))),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    fn fixture() -> Manifest {
        serde_json::from_str(include_str!("../testdata/mojang-manifest.json"))
            .expect("the fixture is a manifest")
    }

    #[test]
    fn the_releases_are_newest_first_without_snapshots_or_alphas() {
        assert_eq!(release_ids(&fixture()), ["26.3", "1.21.4"]);
    }

    #[test]
    fn mojang_failing_or_timing_out_names_it() {
        let unavailable = Ok(Response {
            status: 503,
            headers: BTreeMap::new(),
            body: Vec::new(),
        });
        let timed_out = Err(host::Error::Refused {
            kind: "timed_out".into(),
            message: "the request passed its deadline".into(),
        });
        for sent in [unavailable, timed_out] {
            let err = read::<Manifest>(sent, MANIFEST).expect_err("no catalog");
            assert!(err.downcast_ref::<Refusal>().is_none(), "{err}");
            assert!(err.to_string().contains("Mojang"), "{err}");
        }
    }

    #[test]
    fn a_version_mojang_doesnt_list_is_refused() {
        let manifest = fixture();
        assert!(listed(&manifest, "1.21.4").is_ok());

        let err = listed(&manifest, "9.9.9").expect_err("9.9.9 isn't listed");
        let refusal = err.downcast_ref::<Refusal>().expect("a refusal");
        assert_eq!(refusal.setting, "version");
        assert_eq!(refusal.reason, "Mojang has no version 9.9.9");
    }
}
