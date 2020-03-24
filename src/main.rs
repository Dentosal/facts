#![deny(unused_must_use)]
#![forbid(mutable_borrow_reservation_conflict)]

mod config;
mod dirs;
mod download;
mod error;
mod server;
mod server_process;
mod version;

use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::config::*;
use crate::error::OutputFileAlreadyExists;
use crate::server::Server;

#[cfg(not(unix))]
compile_error!("Non-unix systems are not supported");

/// Global boolean marking whether SIGINT has been detected
static SIGINT: AtomicBool = AtomicBool::new(false);

/// Global boolean marking that SIGINT should not instantly terminate
static SIGINT_CATCH: AtomicBool = AtomicBool::new(false);

#[paw::main]
fn main(args: Args) {
    env_logger::Builder::from_env("FACTS_LOG").init();

    ctrlc::set_handler(move || {
        // SIGINT is automatically transmitted to child processes as well
        log::warn!("Received SIGINT");

        if !SIGINT_CATCH.load(Ordering::SeqCst) {
            std::process::exit(2);
        }

        let not_first = SIGINT.swap(true, Ordering::SeqCst);
        if not_first {
            log::error!("Second SIGINT, abandoning clean up");
            std::process::exit(2);
        }
    })
    .expect("Error setting Ctrl-C handler");

    let result = match args {
        Args::Create { name, config } => cmd_create(&name, config),
        Args::Import {
            name,
            path,
            config,
            meta,
        } => cmd_import(&name, &path, config, meta),
        Args::Export { name, path, force } => cmd_export(&name, &path, force),
        Args::Edit { name, config, meta } => cmd_edit(&name, config, meta),
        Args::Update { name } => cmd_update(&name),
        Args::Delete { name, force } => cmd_delete(&name, force),
        Args::Show { name } => cmd_show(&name),
        Args::List { extended } => cmd_list(extended),
        Args::Prune => cmd_prune(),
        Args::Start { name } => cmd_start(&name),
    };

    match result {
        Ok(()) => {},
        Err(e) => {
            log::error!("{}", e);
            std::process::exit(1);
        },
    }
}

fn cmd_create(name: &str, config: CreateConfig) -> Result<(), Box<dyn std::error::Error>> {
    Server::create(name.to_owned(), config)?;
    Ok(())
}

fn cmd_import(
    name: &str, path: &Path, config: ImportConfig, meta: MetaConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let server = Server::create_empty(name.to_owned(), config, meta)?;
    std::fs::copy(path, server.dir.join("world.zip"))?;
    Ok(())
}

fn cmd_export(name: &str, path: &Path, force: bool) -> Result<(), Box<dyn std::error::Error>> {
    let server = Server::get(name.to_owned())?;

    if path.exists() && !force {
        return Err(Box::new(OutputFileAlreadyExists(path.to_owned())));
    }

    std::fs::copy(server.dir.join("world.zip"), path)?;
    Ok(())
}

fn cmd_edit(
    name: &str, config: ImportConfig, meta: MetaConfigUpdate,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut server = Server::get(name.to_owned())?;
    server.update_config(config, meta)?;
    Ok(())
}

fn cmd_update(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut server = Server::get(name.to_owned())?;
    if let Some(resolved) = server.update_available() {
        server.update(resolved)?;
    }
    Ok(())
}

fn cmd_delete(name: &str, force: bool) -> Result<(), Box<dyn std::error::Error>> {
    let dir = crate::dirs::get_world(&name)?;

    if !force {
        println!(
            "Confirm deletion by typing the name of this server: {}",
            name
        );
        println!("THIS WILL PERMANENTLY DESTROY ALL GAME DATA IN THE SERVER");
        let mut line = String::new();
        std::io::stdin().read_line(&mut line)?;
        if line.trim_end() != name {
            println!("Cancelled");
            std::process::exit(2);
        }
    }

    std::fs::remove_dir_all(dir)?;

    Ok(())
}

fn cmd_show(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let server = Server::get(name.to_owned())?;
    println!("name:       {}", server.name);
    println!("path:       {:?}", server.dir);
    println!("required:   {:?}", server.info.config.factorio);
    println!("current:    {}", server.info.current_version);
    println!("autoupdate: {:?}", server.info.config.autoupdate);
    Ok(())
}

fn cmd_list(extended: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut worlds = dirs::list_worlds();
    worlds.sort();
    for world in worlds {
        if extended {
            let server = Server::get(world.clone())?;
            println!(
                "{:<20} {}  [{}]",
                world, server.info.current_version, server.info.config.factorio
            );
        } else {
            println!("{}", world);
        }
    }
    Ok(())
}

fn cmd_prune() -> Result<(), Box<dyn std::error::Error>> {
    let mut used_versions = HashSet::new();

    for world in dirs::list_worlds() {
        let server = Server::get(world.clone())?;
        used_versions.insert(server.info.current_version);
    }

    for version in dirs::list_versions() {
        if !used_versions.contains(&version) {
            dirs::delete_version(version);
        }
    }

    Ok(())
}

fn cmd_start(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut server = Server::get(name.to_owned())?;
    SIGINT_CATCH.store(true, Ordering::SeqCst);
    server.run()?;
    Ok(())
}
