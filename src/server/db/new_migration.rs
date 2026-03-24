#!/usr/bin/env -S cargo +nightly -Zscript
---cargo
[package]
version = "0.1.0"
edition = "2024"
description = "Tool to generate new migrations"

[dependencies]
clap = { version = "4.6.0", features = ["derive", "string"] }
env_logger = "0.11.8"
log = "0.4.27"
thiserror = "2.0.12"
time = { version = "0.3.47" }
---

use std::env;
use std::ffi::OsString;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::Parser;
use log::info;
use thiserror::Error;
use time::OffsetDateTime;

#[derive(Debug, Parser)]
#[command(name = "Migration tool")]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(default_value_t = String::from(""))]
    migration_action: String,

    #[arg(short, long, default_value = env::current_dir().expect("Valid current directory").into_os_string())]
    base_dir: PathBuf,

    /// Relative to base dir
    #[arg(short = 'd', long, default_value = "migrations")]
    migration_dir: PathBuf,

    /// Relative to migration dir
    #[arg(short = 'm', long, default_value = "mod.rs")]
    migration_module: PathBuf,

    #[arg(short, long, default_value_t = true)]
    format: bool,
}

#[derive(Debug, Error)]
enum MigrationCreateError {
    #[error(transparent)]
    Migration(#[from] MigrationFileError),
    #[error(transparent)]
    Module(#[from] MigrationModUpdateError),
    #[error(transparent)]
    Format(#[from] FormatFileError),
}

fn main() -> Result<(), MigrationCreateError> {
    let cli = Cli::parse();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let migration_dir = PathBuf::from_iter([&cli.base_dir, &cli.migration_dir].iter());

    let migration_module = PathBuf::from_iter([&migration_dir, &cli.migration_module].iter());

    let migration_file_name = format!(
        "m_{timestamp:015}{action}.rs",
        timestamp = OffsetDateTime::now_utc().unix_timestamp(),
        action = if !cli.migration_action.is_empty() {
            format!("_{}", cli.migration_action)
        } else {
            "".into()
        }
    );

    let migration_file_path =
        PathBuf::from_iter([&migration_dir, &migration_file_name.clone().into()].iter());

    write_migration_file(&migration_file_path, &migration_file_name)?;
    update_module(
        &migration_module,
        // remove extension
        &migration_file_name[..(migration_file_name.len() - 3)],
    )?;

    if cli.format {
        format_files(&migration_file_path, &migration_module)?;
    }

    Ok(())
}

#[derive(Debug, Error)]
#[error("Migration file writing failed")]
struct MigrationFileError(#[from] std::io::Error);

fn write_migration_file(
    migration_file_path: &Path,
    migration_file_name: &str,
) -> Result<(), MigrationFileError> {
    info!(
        "Creating migration file at {}",
        migration_file_path.display()
    );

    let mut migration_file = File::create(migration_file_path)?;

    let text = format!(
        r#"use std::sync::Arc;

use libsql::Transaction;

use crate::server::shared::DatabaseError;

pub(super) const MIGRATION_NAME: &str = "{migration_name}";

pub(super) async fn run(db: Arc<Transaction>) -> Result<(), DatabaseError> {{
    todo!()
}}"#,
        migration_name = migration_file_name
    );

    info!("Writing migration content to file");
    migration_file.write_all(text.as_bytes())?;

    Ok(())
}

#[derive(Debug, Error)]
enum MigrationModUpdateError {
    #[error("Migration mod updating failed")]
    File(#[from] std::io::Error),
    #[error("Didn't find extension point \"{0}\" in mod file")]
    MissingLandmark(String),
}

fn update_module(
    migration_mod_path: &Path,
    migration_file_name: &str,
) -> Result<(), MigrationModUpdateError> {
    info!(
        "Reading from module file at {}",
        migration_mod_path.display()
    );
    let mut migration_mod_file_read = OpenOptions::new().read(true).open(migration_mod_path)?;

    let mut file_string = String::new();
    migration_mod_file_read.read_to_string(&mut file_string)?;
    drop(migration_mod_file_read);

    let Some(mod_end) = file_string.find("// migration_import_end") else {
        return Err(MigrationModUpdateError::MissingLandmark(
            "// migration_import_end".into(),
        ));
    };
    let Some(migrations_end) = file_string.find("// migration_list_end") else {
        return Err(MigrationModUpdateError::MissingLandmark(
            "// migration_list_end".into(),
        ));
    };

    // In reverse so indices remain valid
    info!("Adding migration to migration list");
    let migration_list_string = format!(
        r#"Migration {{
            name: {migration_file_name}::MIGRATION_NAME,
            run: Box::new(|db: Arc<Transaction>| Box::pin({migration_file_name}::run(db))),
        }},
        "#,
        migration_file_name = migration_file_name
    );
    file_string.insert_str(migrations_end, &migration_list_string);

    info!("Adding migration import");
    let migration_import_string = format!(
        "mod {migration_file_name};\n",
        migration_file_name = migration_file_name
    );
    file_string.insert_str(mod_end, &migration_import_string);

    info!("Writing changes to file");
    let mut migration_mod_file_write = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(migration_mod_path)?;

    migration_mod_file_write.write_all(file_string.as_bytes())?;
    migration_mod_file_write.flush()?;

    Ok(())
}

#[derive(Debug, Error)]
enum FormatFileError {
    #[error("Formatting files failed")]
    Io(#[from] std::io::Error),

    #[error("Formatting files failed: {}", .0.display())]
    Command(OsString),
}

fn format_files(
    migration_file_path: &Path,
    migration_mod_path: &Path,
) -> Result<(), FormatFileError> {
    let mut command = OsString::from("rustfmt --edition 2024 ");
    command.push(migration_file_path.as_os_str());
    command.push(" ");
    command.push(migration_mod_path.as_os_str());

    info!("Formating files");
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd").arg("/C").arg(command).output()?
    } else {
        Command::new("sh").arg("-c").arg(command).output()?
    };

    if !output.status.success() {
        let mut error_string = OsString::new();

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::ffi::OsStringExt;

            error_string.push(OsString::from_wide(
                output.stderr.into_iter().map(|n| n as u16).collect(),
            ));
        }

        #[cfg(target_family = "unix")]
        {
            use std::os::unix::ffi::OsStringExt;

            error_string.push(OsString::from_vec(output.stderr));
        }

        return Err(FormatFileError::Command(error_string));
    }

    Ok(())
}
