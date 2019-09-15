use std::process::Command;
use std::path::PathBuf;
use crate::services::Repository;
use crate::error::Error;

pub struct Opts<'a> {
    pub repo_url: &'a str,
    pub username: &'a str,
    pub password: &'a str,
    pub destination: PathBuf
}

pub fn sync_repository(opts: Opts) -> Result<(),Error> {

    // Create the destination folder:
    std::fs::create_dir_all(&opts.destination).map_err(|e|
        err!("Could not create path '{}': {}", opts.destination.to_string_lossy(), e)
    )?;

    // Is the folder already a bare repo? It is if
    // it contains a file called HEAD.
    let mut dest_head = opts.destination.clone();
    dest_head.push("HEAD");
    let is_repo = dest_head.is_file();

    // Sync or clone depending on whether already a repo:
    let output = if is_repo {
        Command::new("sh")
            .arg("-c").arg(git_fetch_cmd(opts.repo_url))
            .env("GIT_USER", opts.username)
            .env("GIT_PASSWORD", opts.password)
            .current_dir(opts.destination)
            .output()?
    } else {
        Command::new("sh")
            .arg("-c").arg(git_clone_cmd(opts.repo_url))
            .env("GIT_USER", opts.username)
            .env("GIT_PASSWORD", opts.password)
            .current_dir(opts.destination)
            .output()?
    };

    Ok(())
}

fn git_clone_cmd(repo_url: &str) -> String {
    let mut cmd = String::from(r#"
        git clone \
            --bare \
            --config credential.helper='!f() { sleep 1; echo "username=${GIT_USER}"; echo "password=${GIT_PASSWORD}"; }; f' \
    "#);
    // repo to clone:
    cmd.push_str(repo_url);
    // clone into current directory:
    cmd.push_str(" .");
    cmd
}

fn git_fetch_cmd(repo_url: &str) -> String {
    let mut cmd = String::from("git fetch origin '*:*' --prune --prune-tags ");
    cmd.push_str(repo_url);
    cmd
}
