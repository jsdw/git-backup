#[macro_use]
mod error;
mod services;
mod git;

use error::Error;
use std::path::PathBuf;
use structopt::StructOpt;
use services::{ Github, Service };

#[derive(StructOpt, Debug)]
#[structopt(name = "git-backup")]
struct Opts {
    /// URL of repositories to backup
    #[structopt(name="source")]
    url: String,
    /// Location to place the backups. If not provided,
    /// the current working directory will be used
    #[structopt(name="destination", parse(from_os_str))]
    backup_location: Option<PathBuf>,
    /// Only backup public respositories (may not need a token)
    #[structopt(long="public-only")]
    public: bool,
    /// An access token for the service you're trying to backup from.
    #[structopt(short="t", long="token")]
    token: String
}

fn main() -> Result<(),Error> {
    let opts = Opts::from_args();

    println!("{:#?}", opts);

    let dest_path = opts.backup_location
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let service: Option<Box<dyn Service>> =
        if let Some(gh) = Github::new(opts.url, Some(opts.token.clone()), opts.public) {
            Some(Box::new(gh))
        } else {
            None
        };

    let service = service.ok_or_else(|| err!("URL not recognised"))?;
    let repos = service.list_repositories()?;
    let username = service.username();

    println!("Backing up {} repos", repos.len());

    for repo in repos {
        println!("syncing {}", repo.name);
        let mut repo_path = dest_path.clone();
        repo_path.push(repo.name.clone());
        git::sync_repository(git::Opts {
            repo_url: &repo.git_url,
            username: &username,
            password: &opts.token,
            destination: repo_path
        })?;
    }

    Ok(())
}
