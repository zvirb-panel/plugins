//! The Java runtime a server runs on.

use zvirb_pdk::Error;

/// The `eclipse-temurin` JRE image per Java major, pinned to the digest of
/// its multi-arch index so every host runs the same bytes (D-0048). The
/// digests were read from Docker Hub on 2026-11-16; a newer one ships as a
/// plugin release.
const IMAGES: &[(u32, &str)] = &[
    (
        8,
        "eclipse-temurin:8-jre@sha256:d1450af461d0d791f3b405895baf11ad37881a66e9a4dd9bbe2cd10d99278caf",
    ),
    (
        11,
        "eclipse-temurin:11-jre@sha256:7db5776af2396377c8aae2aa124712da7b72a5b6b014c17d77ae37eb5276f37b",
    ),
    (
        17,
        "eclipse-temurin:17-jre@sha256:fb2cec8da84e3f086b59e29bf432d372e396fe0c247adcfb89303a28c2e313c0",
    ),
    (
        21,
        "eclipse-temurin:21-jre@sha256:ca73b4d3961703d6d7830a4361f49ef0478f8f28b39defc07bb72f4cc7f834f6",
    ),
    (
        25,
        "eclipse-temurin:25-jre@sha256:945247f9d092703e0c6be92243ef3c51640229e23ab34d353c8dcdc73e6cd4e6",
    ),
];

/// The image of the oldest Java major at or above `minimum`. Java runs code
/// built for an older major, so a server that asks for 16, which Temurin no
/// longer publishes, runs on 17.
pub fn image(minimum: u32) -> Result<&'static str, Error> {
    IMAGES
        .iter()
        .find(|(major, _)| *major >= minimum)
        .map(|(_, image)| *image)
        .ok_or_else(|| {
            Error::msg(format!(
                "the server needs Java {minimum}, newer than this plugin's images"
            ))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn takes_the_oldest_major_at_or_above_the_minimum() {
        assert!(
            image(21)
                .expect("21 has an image")
                .starts_with("eclipse-temurin:21-jre@")
        );
        assert!(
            image(16)
                .expect("17 covers 16")
                .starts_with("eclipse-temurin:17-jre@")
        );
        assert!(
            image(1)
                .expect("8 covers older")
                .starts_with("eclipse-temurin:8-jre@")
        );
    }

    #[test]
    fn refuses_a_major_newer_than_every_image() {
        assert!(image(26).is_err());
    }
}
