#[macro_use]
mod error;
#[macro_use]
mod logging;
mod services;
mod git;

use error::Error;
use rayon::prelude::*;
use std::path::PathBuf;
use std::collections::HashSet;
use structopt::StructOpt;
use services::{ Github, GitLab, Bitbucket, Service };

#[derive(StructOpt, Debug)]
#[structopt(name = "git-backup", author = "James Wilson <james@jsdw.me>")]
struct Opts {
    /// URL of repositories to backup
    #[structopt(name="source")]
    url: String,
    /// Location to place the backups. If not provided,
    /// the current working directory will be used
    #[structopt(name="destination", parse(from_os_str))]
    backup_location: Option<PathBuf>,
    /// An access token for the service you're trying to backup from.
    /// this can be provided via the environment variable GIT_TOKEN
    /// instead, but is required in one of those forms.
    #[structopt(long="token")]
    token: Option<String>,
    /// Remove folders in the destination that don't correspond to
    /// repositories that we have found to back up.
    #[structopt(long="prune")]
    prune: bool,
    /// Don't actually back anything up; just log what we'll do.
    #[structopt(long="dry-run")]
    dry_run: bool
}

fn main() {
    if let Err(e) = run() {
        log_error!("{}", e);
    }
}

fn run() -> Result<(),Error> {

    // Check that we have a valid version of git installed:
    let git_version = git::version().map_err(|_| err!("Git does not appear to be installed"))?;
    if git_version < git::Version::new(2,0,0) {
        return Err(err!("Your version of git appears to be too old. This command requires at least 2.0.0"))
    }

    // Prepare our options:
    let opts = Opts::from_args();
    let dry_run = opts.dry_run;
    let prune = opts.prune;
    let url = opts.url;
    let token = opts.token
        .or_else(|| std::env::var("GIT_TOKEN").ok())
        .ok_or_else(|| err!("Need either --token or GIT_TOKEN env var to be provided"))?;
    let dest_path = opts.backup_location
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    // Find a matching service:
    let service = pick_service(url.clone(), token.clone())
        .ok_or_else(|| err!("Source '{}' not recognised", &url))?;
    let repos = service.list_repositories()?;
    let username = service.username();

    if repos.len() != 1 {
        log_info!("Backing up {} repositories", repos.len());
    } else {
        log_info!("Backing up 1 repository");
    }

    // Perform the backup:
    repos.par_iter().for_each(|repo| {
        log_info!("Syncing '{}'", repo.name);
        let mut repo_path = dest_path.clone();
        repo_path.push(repo_name_to_folder(&repo.name));

        if !dry_run {
            let sync_result = git::sync_repository(git::Opts {
                repo_url: &repo.git_url,
                username: &username,
                password: &token,
                destination: &repo_path
            });
            if let Err(e) = sync_result {
                log_error!("Could not sync repository '{}': \n{}", repo_path.to_string_lossy(), e);
            }
        }

    });

    // Prune folders that may have been created with this app
    // from a prior backup but are now no logner needed.
    if prune {
        let keep_these_folders: HashSet<String> = repos
            .into_iter()
            .map(|repo| repo_name_to_folder(&repo.name))
            .collect();
        for entry in std::fs::read_dir(dest_path)? {
            // Ignore things we run into an issue reading:
            let entry = if let Ok(entry) = entry {
                entry
            } else {
                continue
            };
            // Ignore non-directories:
            if !entry.path().is_dir() {
                continue;
            }
            // Ignore non-utf8 filenames (this program wouldn't have created them):
            let file_name = if let Ok(name) = entry.file_name().into_string() {
                name
            } else {
                continue
            };
            // Ignore filenames not ending in '.git':
            if !file_name.ends_with(".git") {
                continue
            }
            // Ignore filenames for current repos:
            if keep_these_folders.contains(&file_name) {
                continue
            }
            // Remove the folder and its contents (if not dry_run):
            log_info!("Pruning {}", file_name);
            if !dry_run {
                if let Some(err) = std::fs::remove_dir_all(entry.path()).err() {
                    log_error!("Error pruning {}: {}", file_name, err);
                }
            }
        }
    }

    log_info!("Backup completed!");

    Ok(())
}

fn repo_name_to_folder(repo_name: &str) -> String {
    format!("{}.git", repo_name)
}

fn pick_service(url: String, token: String) -> Option<Box<dyn Service>> {
    if let Some(gh) = Github::new(
        url.clone(),
        Some(token.clone())
    ) {
        Some(Box::new(gh))
    } else if let Some(bb) = Bitbucket::new(
        url.clone(),
        Some(token.clone())
    ) {
        Some(Box::new(bb))
    } else if let Some(gl) = GitLab::new(
        url.clone(),
        Some(token.clone())
    ) {
        Some(Box::new(gl))
    } else {
        None
    }
}
