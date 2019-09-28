#[macro_use]
mod error;
mod services;
mod git;

use error::Error;
use rayon::prelude::*;
use std::path::PathBuf;
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
    token: Option<String>
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
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

    // Perform the backup:
    println!("Backing up {} repos", repos.len());
    repos.into_par_iter().for_each(|repo| {
        println!("syncing {}", repo.name);
        let mut repo_path = dest_path.clone();
        repo_path.push(format!("{}.git", repo.name));

        let sync_result = git::sync_repository(git::Opts {
            repo_url: &repo.git_url,
            username: &username,
            password: &token,
            destination: &repo_path
        });
        if let Err(e) = sync_result {
            eprintln!("Error syncing repository '{}': {}", repo_path.to_string_lossy(), e);
        }

    });
    println!("Backup completed!");

    Ok(())
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