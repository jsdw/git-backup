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
    let mut dest = opts.destination;
    dest.push("HEAD");

    // Sync or clone depending on whether already a repo:
    let output = if dest.is_file() {
        Command::new("sh")
            .arg("-c").arg(git_fetch_cmd(opts.repo_url))
            .env("GIT_USER", opts.username)
            .env("GIT_PASSWORD", opts.password)
            .output()?
    } else {
        Command::new("sh")
            .arg("-c").arg(git_clone_cmd(opts.repo_url))
            .env("GIT_USER", opts.username)
            .env("GIT_PASSWORD", opts.password)
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
    cmd.push_str(repo_url);
    cmd
}

fn git_fetch_cmd(repo_url: &str) -> String {
    let mut cmd = String::from("git fetch --prune --force");
    cmd.push_str(repo_url);
    cmd
}

// // The following clones a repo (even a private one) using the HTTPS URL
// // and using the private access token for auth, so no need for separate
// // SSH keys setup. The credentials helper is kept in config so we can reuse
// // for updating the repo easily.
//
// export GIT_USER=username
// export GIT_PASSWORD=personalAccessToken
// git clone \
//     --bare \
//     --config credential.helper='!f() { sleep 1; echo "username=${GIT_USER}"; echo "password=${GIT_PASSWORD}"; }; f' \
//     https://github.com/username/repo
//
// // To update the bare repo from some URL (above env vars needed) (do we need --all?):
//
// git fetch --prune https://github.com/username/repo