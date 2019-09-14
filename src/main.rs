#[macro_use]
mod error;
mod services;

use error::Error;
use std::path::PathBuf;
use structopt::StructOpt;
use services::{ Github, Service };

#[derive(StructOpt, Debug)]
#[structopt(name = "git-backup")]
struct Opts {
    /// URL of repositories to back up
    #[structopt(name="URL")]
    url: String,
    /// Location to place the backups. If not provided,
    /// the current working directory will be used
    #[structopt(short="l", long="location", parse(from_os_str))]
    backup_location: Option<PathBuf>,
    /// Only backup public respositories (may not need a token)
    #[structopt(long="public-only")]
    public: bool,
    /// An access token for the service you're trying to backup
    /// from. This is necessary to be able to read private repositories.
    #[structopt(short="t", long="token")]
    token: Option<String>
}

fn main() -> Result<(),Error> {
    let opts = Opts::from_args();
    println!("{:#?}", opts);

    let service: Option<Box<dyn Service>> =
        if let Some(gh) = Github::new(opts.url, opts.token, opts.public) {
            Some(Box::new(gh))
        } else {
            None
        };

    let service = service.ok_or_else(|| err!("URL not recognised"))?;

    let repos = service.list_repositories()?;

    println!("Repos: {:?}", repos);
    println!("Backing up {} repos", repos.len());

    Ok(())
}
