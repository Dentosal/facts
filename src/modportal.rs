use reqwest::{
    blocking::{Client, ClientBuilder},
    header, StatusCode,
};
use serde_json::json;
use std::fs::File;
use std::io::copy;
use std::path::PathBuf;

use crate::config::{LoginCredentials, TokenCredentials};
use crate::dirs;
use crate::error::{
    InternalDataModified, LoginFailed, NoMatchingModVersions, NoSuchMod, NotLoggedIn,
};
use crate::version::{Version, Version2};

const INVALID_DATA: &str = "Invalid response from factorio API";

#[derive(Debug)]
pub struct ModInfo {
    pub name: String,
    pub version: Version,
}
impl ModInfo {
    fn from_file_name(s: &str) -> Option<Self> {
        let mut it = s.rsplitn(2, '.');
        if it.next()? != "zip" {
            return None;
        }

        let stem = it.next()?;
        let mut it = stem.rsplitn(2, '_');

        let version = Version::try_from_str(it.next()?).ok()?;
        let name = it.next()?.to_owned();

        Some(Self { name, version })
    }

    pub fn try_from_file_name(s: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let error = Box::new(InternalDataModified("mod file name".to_owned()));
        Self::from_file_name(s).ok_or(error)
    }

    pub fn file_name(&self) -> String {
        format!("{}_{}.zip", app_dirs::sanitized(&self.name), self.version)
    }

    pub fn path(&self) -> PathBuf {
        let mut pb = dirs::app_root();
        pb.push("mods");
        pb.push(self.file_name());
        pb
    }
}

pub struct ModDownloader {
    client: Client,
    credentials: TokenCredentials,
}
impl ModDownloader {
    pub fn new() -> Result<Self, NotLoggedIn> {
        if let Some(credentials) = TokenCredentials::load() {
            Ok(Self {
                client: ClientBuilder::new().cookie_store(true).build().unwrap(),
                credentials,
            })
        } else {
            Err(NotLoggedIn)
        }
    }

    /// Downloads latest matching version
    pub fn require(
        &self, name: &str, game_version: Version,
    ) -> Result<ModInfo, Box<dyn std::error::Error>> {
        let (mod_info, download_link) = latest_version(&self.client, name, game_version)?;

        dirs::create_mods_dir();
        if mod_info.path().exists() {
            return Ok(mod_info);
        }

        self.download_mod(&mod_info, &download_link)?;
        Ok(mod_info)
    }

    fn download_mod(
        &self, mod_info: &ModInfo, url: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut r = self
            .client
            .get(&format!("https://mods.factorio.com{}", url))
            .query(&json!({
                "username": self.credentials.username.clone(),
                "token": self.credentials.token.plaintext.clone()
            }))
            .send()?;

        if r.headers()[header::CONTENT_TYPE]
            .to_str()
            .unwrap()
            .starts_with("text/html")
        {
            return Err(Box::new(NotLoggedIn));
        }

        let mut f = File::create(&mod_info.path())?;
        copy(&mut r, &mut f)?;
        Ok(())
    }

    pub fn login(&self, credentials: LoginCredentials) -> Result<(), Box<dyn std::error::Error>> {
        let resp = self
            .client
            .post("https://auth.factorio.com/api-login")
            .form(&json!({
                "username": credentials.username.expect("Username required"),
                "password": credentials.password.expect("Password required").plaintext,
                "api_version": 2,
                "require_game_ownership": true
            }))
            .send()?;

        if !resp.status().is_success() {
            return Err(Box::new(LoginFailed(
                resp.json::<serde_json::Value>().unwrap()["message"]
                    .as_str()
                    .unwrap()
                    .to_owned(),
            )));
        }

        let cred: TokenCredentials = resp.json().unwrap();
        cred.store();

        Ok(())
    }
}

mod api {
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct Mod {
        pub name: String,
        pub releases: Vec<ModRelease>,
    }

    #[derive(Debug, Deserialize)]
    pub struct ModRelease {
        pub download_url: String,
        pub version: String,
        pub info_json: ModReleaseInfoJson,
    }
    #[derive(Debug, Deserialize)]
    pub struct ModReleaseInfoJson {
        pub factorio_version: String,
    }
}

/// Resolves latest matching version
fn latest_version(
    client: &Client, name: &str, game_version: Version,
) -> Result<(ModInfo, String), Box<dyn std::error::Error>> {
    let error = Box::new(NoMatchingModVersions(name.to_owned(), game_version));

    let resp = client
        .get(&format!("https://mods.factorio.com/api/mods/{}", name))
        .send()?;

    if resp.status() == StatusCode::NOT_FOUND {
        return Err(Box::new(NoSuchMod(name.to_owned())));
    }

    resp.json::<api::Mod>()?
        .releases
        .iter()
        .filter(|r| {
            Version2::try_from_str(&r.info_json.factorio_version)
                .expect(INVALID_DATA)
                .includes(game_version)
        })
        .last()
        .map(|r| {
            (
                ModInfo {
                    name: name.to_owned(),
                    version: Version::try_from_str(&r.version).expect(INVALID_DATA),
                },
                r.download_url.clone(),
            )
        })
        .ok_or(error)
}

#[derive(Deserialize)]
pub struct ModListJson {
    mods: Vec<ModListJsonMod>,
}

#[derive(Deserialize)]
pub struct ModListJsonMod {
    name: String,
    enabled: bool,
}

pub fn load_mod_list_json(path: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mod_list: ModListJson = serde_json::from_slice(&fs::read(path)?)?;
    Ok(mod_list
        .mods
        .into_iter()
        .filter_map(|m| {
            if m.name != "base" && m.enabled {
                Some(m.name)
            } else {
                None
            }
        })
        .collect())
}
