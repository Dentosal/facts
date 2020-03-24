//! Builds file configuration for a server

use crossbeam_channel::unbounded;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::Ordering;
use std::thread::{self, JoinHandle};

use crate::config::*;
use crate::download;
use crate::error::DowngradingNotAllowed;
use crate::server_process::{self, message};
use crate::version::{ResolvedVersionReq, Version};

/// Server info data version format
const SERVER_INFO_VERSION: u64 = 1;

/// Server data to persist to disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    _version: u64,
    pub config: MetaConfig,
    pub current_version: Version,
}

#[derive(Debug)]
pub struct Server {
    pub name: String,
    pub dir: PathBuf,
    pub info: ServerInfo,
}
impl Server {
    /// Creates a new server from name and config
    pub fn try_new(name: String, config: CreateConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let dir = crate::dirs::new_world(&name)?;
        let current_version = download::require(config.meta.factorio.clone())?;

        let s = Self {
            dir,
            name,
            info: ServerInfo {
                _version: SERVER_INFO_VERSION,
                config: config.meta.clone(),
                current_version,
            },
        };

        s.create_config_ini();
        s.create_copy_files(&config);
        s.save();

        Ok(s)
    }

    /// Loads server configuration by name
    pub fn get(name: String) -> Result<Self, Box<dyn std::error::Error>> {
        let dir = crate::dirs::get_world(&name)?;

        let contents = fs::read_to_string(dir.join("facts.json"))
            .expect("Could not read server configuration");
        let info: ServerInfo = serde_json::from_str(&contents).expect("Invalid JSON");

        assert_eq!(
            info._version, SERVER_INFO_VERSION,
            "Unsupported server info version"
        );

        Ok(Self { dir, name, info })
    }

    /// Updates server config from an ImportConfig
    pub fn update_config(
        &mut self, config: ImportConfig, meta: MetaConfigUpdate,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.import_copy_files(&config);
        self.info.config.apply_update(meta);

        if let Some(resolved) = self.latest_version() {
            if resolved.version < self.info.current_version {
                return Err(Box::new(DowngradingNotAllowed {
                    current: self.info.current_version,
                    requested: self.info.config.factorio.clone(),
                }));
            }
            self.update(resolved)?;
        } else {
            self.save();
        }
        Ok(())
    }

    /// Saves server configuration
    pub fn save(&self) {
        fs::write(
            self.dir.join("facts.json"),
            serde_json::to_string(&self.info).unwrap(),
        )
        .expect("Could not write server info");
    }

    /// Create config.ini file to force server
    fn create_config_ini(&self) {
        fs::write(
            self.dir.join("config.ini"),
            format!(
                "[path]\nread-data=__PATH__executable__/../../data\nwrite-data={}\n",
                self.dir.to_str().unwrap()
            ),
        )
        .expect("Could not write config.ini")
    }

    /// Copy file into world folder
    fn copy_file(&self, path: &Path, name: &str) {
        fs::copy(path, self.dir.join(name)).expect("Could not copy file");
    }

    /// Copy settings files into the world directory
    fn create_copy_files(&self, config: &CreateConfig) {
        if let Some(path) = &config.map_gen_settings {
            self.copy_file(path, "map-gen-settings.json");
        }

        if let Some(path) = &config.map_settings {
            self.copy_file(path, "map-settings.json")
        }

        self.import_copy_files(&config.import);
    }

    /// Copy settings files into the world directory
    pub fn import_copy_files(&self, config: &ImportConfig) {
        if let Some(path) = &config.server_settings {
            self.copy_file(path, "server-settings.json");
        }

        let mut admins: Vec<String> = config.add_admin.clone();
        if let Some(path) = &config.server_adminlist {
            let content = fs::read_to_string(path).expect("Could not read file");
            let file_admins: Vec<String> = serde_json::from_str(&content).expect("Invalid JSON");
            admins.extend(file_admins);
        }
        fs::write(
            self.dir.join("server-adminlist.json"),
            serde_json::to_string(&admins).unwrap(),
        )
        .expect("Could not write file");
    }

    fn command_base(&self) -> Command {
        let mut cmd = Command::new(
            self.info
                .current_version
                .location()
                .expect("Currect Factorio version missing from downloads")
                .join("factorio/bin/x64/factorio"),
        );
        cmd.current_dir(&self.dir);
        cmd
    }

    /// Generate world based on the settings
    fn generate_args(&self) -> Vec<&str> {
        let mut args = Vec::new();
        args.push("--config");
        args.push("config.ini");
        args.push("--create");
        args.push("world");
        if self.dir.join("map-gen-settings.json").exists() {
            args.push("--map-gen-settings");
            args.push("map-gen-settings.json");
        }
        if self.dir.join("map-settings.json").exists() {
            args.push("--map-settings");
            args.push("map-settings.json");
        }

        args
    }

    /// Generate world based on the settings
    pub fn generate(&self) {
        log::info!("Generating world");

        let output = self
            .command_base()
            .args(self.generate_args())
            .output()
            .unwrap();

        if !output.status.success() {
            println!("{}", String::from_utf8(output.stdout).unwrap());
            panic!("World generation failed");
        }

        log::info!("Done");
    }

    fn start_args(&self) -> Vec<&str> {
        let mut args = Vec::new();
        args.push("--config");
        args.push("config.ini");
        args.push("--start-server");
        args.push("world.zip");
        args.push("--server-adminlist");
        args.push("server-adminlist.json");
        if self.dir.join("server-settings.json").exists() {
            args.push("--server-settings");
            args.push("server-settings.json");
        }

        args
    }

    fn latest_version(&self) -> Option<ResolvedVersionReq> {
        match self.info.config.factorio.resolve() {
            Ok(latest) => Some(latest),
            Err(error) => {
                log::warn!("Could not check for updates: {}", error);
                None
            },
        }
    }

    pub fn update_available(&self) -> Option<ResolvedVersionReq> {
        if let Some(latest) = self.latest_version() {
            if latest.version > self.info.current_version {
                log::trace!("Update available: {}", latest.version);
                Some(latest)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Update server to given version
    pub fn update(
        &mut self, resolved: ResolvedVersionReq,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Updating server to {}", resolved.version);

        self.info.current_version = download::require_resolved(resolved)?;
        self.save();

        log::info!("Server updated");

        Ok(())
    }

    /// Returns Ok(Some) to request update and restart, and Ok(None) to shutdown
    fn run_once(&self) -> Result<Option<ResolvedVersionReq>, Box<dyn std::error::Error>> {
        log::info!("Starting server {}", self.name);

        let child = self
            .command_base()
            .args(self.start_args())
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();

        let (tx_to, rx_to) = unbounded::<message::ToServer>();
        let (tx_from, rx_from) = unbounded::<message::FromServer>();

        let handle: JoinHandle<Result<(), _>> =
            thread::spawn(move || server_process::run(child, tx_from, rx_to));

        let msg = rx_from.recv().expect("Server thread crashed");
        assert!(matches!(msg, message::FromServer::StartupComplete));

        log::info!("Server is running");

        let mut result = None;
        if self.info.config.autoupdate.live() {
            'outer: loop {
                let sleep_ms: u64 = 60 * 1000 * self.info.config.autoupdate_interval_minutes;
                let interval: u64 = 50;
                for _ in (0..sleep_ms).step_by(interval as usize) {
                    if crate::SIGINT.load(Ordering::SeqCst) {
                        result = None;
                        break 'outer;
                    }
                    thread::sleep(std::time::Duration::from_millis(interval));
                }

                if let Some(resolved) = self.update_available() {
                    if self.info.config.autoupdate == AutoUpdate::Forced {
                        log::warn!("Autoupdate: restarting server");
                        tx_to
                            .send(message::ToServer::Shutdown)
                            .expect("Server thread crashed");
                        result = Some(resolved);
                        break;
                    } else {
                        assert_eq!(self.info.config.autoupdate, AutoUpdate::Enabled);
                        tx_to
                            .send(message::ToServer::GetState)
                            .expect("Server thread crashed");

                        let reply = rx_from.recv().expect("Server thread crashed");
                        if let message::FromServer::State(state) = reply {
                            if state.players_online.is_empty() {
                                log::warn!("Autoupdate: restarting server");
                                tx_to
                                    .send(message::ToServer::Shutdown)
                                    .expect("Server thread crashed");
                                result = Some(resolved);
                                break;
                            } else {
                                log::trace!("Not updating server as there are players online");
                            }
                        } else {
                            unreachable!("Wrong response type received");
                        }
                    }
                }
            }
        }

        handle.join().expect("Server thread crashed")?;
        Ok(result)
    }

    /// Run the server
    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.info.config.autoupdate != AutoUpdate::Disabled {
            if let Some(resolved) = self.update_available() {
                self.update(resolved)?;
            }
        }

        while let Some(resolved) = self.run_once()? {
            self.update(resolved)?;
        }

        Ok(())
    }
}
