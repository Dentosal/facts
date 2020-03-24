use scraper::{Html, Selector};
use serde_json::Value;
use std::path::Path;
use tar::Archive;
use xz2::read::XzDecoder;

use crate::error::NoDownloadAvailable;
use crate::version::{ResolvedVersionReq, Version, VersionReq};

const INVALID_DATA: &str = "Invalid response from factorio API";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LatestReleases {
    pub experimental: Version,
    pub stable: Version,
}
impl LatestReleases {
    pub fn get() -> Result<Self, Box<dyn std::error::Error>> {
        let resp =
            reqwest::blocking::get("https://factorio.com/api/latest-releases")?.json::<Value>()?;

        log::trace!("Requesting latest release numbers");

        let s = Self {
            experimental: Version::from_str(
                &resp
                    .get("experimental")
                    .expect(INVALID_DATA)
                    .get("headless")
                    .expect(INVALID_DATA)
                    .as_str()
                    .expect(INVALID_DATA),
            ),
            stable: Version::from_str(
                &resp
                    .get("stable")
                    .expect(INVALID_DATA)
                    .get("headless")
                    .expect(INVALID_DATA)
                    .as_str()
                    .expect(INVALID_DATA),
            ),
        };

        log::trace!("Latest releases {:?}", s);

        Ok(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Release {
    pub version: Version,
}
impl Release {
    pub fn url(&self) -> String {
        format!(
            "https://www.factorio.com/get-download/{}/headless/linux64",
            self.version
        )
    }

    fn get_all_from(url: &str) -> Result<Vec<Self>, Box<dyn std::error::Error>> {
        let resp = reqwest::blocking::get(url)?;
        let document = Html::parse_document(&resp.text()?);
        let selector = Selector::parse("h3").unwrap();
        Ok(document
            .select(&selector)
            .map(|x| Self {
                version: Version::from_str(
                    &x.text()
                        .next()
                        .expect(INVALID_DATA)
                        .split_whitespace()
                        .next()
                        .expect(INVALID_DATA)
                        .to_owned(),
                ),
            })
            .collect())
    }

    pub fn get_stables() -> Result<Vec<Self>, Box<dyn std::error::Error>> {
        Self::get_all_from("https://www.factorio.com/download-headless")
    }

    pub fn get_experimentals() -> Result<Vec<Self>, Box<dyn std::error::Error>> {
        Self::get_all_from("https://www.factorio.com/download-headless/experimental")
    }

    pub fn get_all_by_hint(
        stability_hint: Option<bool>,
    ) -> Result<Vec<Self>, Box<dyn std::error::Error>> {
        let mut result: Vec<Self> = Vec::new();
        if stability_hint != Some(true) {
            result.extend(Self::get_experimentals()?.into_iter());
        }
        if stability_hint != Some(false) {
            result.extend(Self::get_stables()?.into_iter());
        }
        Ok(result)
    }
}

/// Downloads requested version if not already available
pub fn require(version_req: VersionReq) -> Result<Version, Box<dyn std::error::Error>> {
    require_resolved(version_req.resolve()?)
}

/// Downloads requested version if not already available
pub fn require_resolved(
    resolved: ResolvedVersionReq,
) -> Result<Version, Box<dyn std::error::Error>> {
    match resolved.version.location() {
        Ok(_location) => {
            log::info!("Factorio {} already downloaded", resolved.version);
            Ok(resolved.version)
        },
        Err(target_path) => {
            let releases = Release::get_all_by_hint(resolved.stability_hint)?;

            let mut url: Option<String> = None;
            for r in releases {
                if r.version == resolved.version {
                    url = Some(r.url());
                    break;
                }
            }

            if let Some(url) = url {
                log::info!("Downloading Factorio {}", resolved.version);
                download_version(&target_path, &url)?;
                log::info!("Download complete");
                Ok(resolved.version)
            } else {
                Err(Box::new(NoDownloadAvailable(resolved.version)))
            }
        },
    }
}

fn download_version(target_path: &Path, url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let response = reqwest::blocking::get(url)?;

    assert!(
        response
            .headers()
            .get("content-disposition")
            .expect(INVALID_DATA)
            .to_str()
            .expect(INVALID_DATA)
            .ends_with(".tar.xz"),
        INVALID_DATA
    );

    let mut archive = Archive::new(XzDecoder::new(response));
    archive
        .unpack(target_path)
        .expect("Unable to unpack archive");

    Ok(())
}
