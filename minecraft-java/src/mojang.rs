//! Mojang's version manifest, as the Vanilla engine reads it.

use serde::Deserialize;
use serde::de::DeserializeOwned;
use zvirb_pdk::Error;
use zvirb_pdk::engine::Refusal;
use zvirb_pdk::host::http::{self, Method, Request};

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
    pub url: String,
}

/// A version's own file: what it runs on and what it downloads.
#[derive(Debug, Deserialize)]
pub struct Version {
    #[serde(rename = "javaVersion")]
    pub java: Option<Java>,
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

fn listed<'a>(manifest: &'a Manifest, version: &str) -> Result<&'a Listed, Error> {
    manifest
        .versions
        .iter()
        .find(|listed| listed.id == version)
        .ok_or_else(|| Refusal::new("version", format!("Mojang has no version {version}")).into())
}

fn get<T: DeserializeOwned>(url: &str) -> Result<T, Error> {
    let response = http::send(&Request::new(Method::Get, url))?;
    match response.status {
        200 => response
            .json()
            .map_err(|err| Error::msg(format!("read Mojang's answer from {url}: {err}"))),
        status => Err(Error::msg(format!("Mojang's {url} answered {status}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_version_mojang_doesnt_list_is_refused() {
        let manifest: Manifest =
            serde_json::from_str(include_str!("../testdata/mojang-manifest.json"))
                .expect("the fixture is a manifest");
        assert!(listed(&manifest, "1.21.4").is_ok());

        let err = listed(&manifest, "9.9.9").expect_err("9.9.9 isn't listed");
        let refusal = err.downcast_ref::<Refusal>().expect("a refusal");
        assert_eq!(refusal.setting, "version");
        assert_eq!(refusal.reason, "Mojang has no version 9.9.9");
    }
}
