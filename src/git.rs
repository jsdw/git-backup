use regex::Regex;
use lazy_static::lazy_static;
use std::process::Command;
use std::path::Path;
use crate::error::Error;

#[derive(Debug,Clone,Copy,PartialOrd,Ord,PartialEq,Eq)]
pub struct Version {
    major: u8,
    minor: u8,
    patch: u8
}

impl Version {
    pub fn new(major: u8, minor: u8, patch: u8) -> Version {
        Version { major, minor, patch }
    }
}

pub fn version() -> Result<Version,Error> {
    lazy_static! {
        static ref GIT_VERSION_RE: Regex = Regex::new("([0-9]+)\\.([0-9]+)\\.([0-9]+)").unwrap();
    }
    let out = Command::new("sh")
        .arg("-c").arg("git version")
        .output()?;

    let stdout = String::from_utf8_lossy(&out.stdout).to_owned();
    let caps = GIT_VERSION_RE.captures(&stdout).ok_or_else(|| err!("Cannot parse version from {}", &stdout))?;

    let major = caps.get(1).unwrap().as_str().parse().unwrap();
    let minor = caps.get(2).unwrap().as_str().parse().unwrap();
    let patch = caps.get(3).unwrap().as_str().parse().unwrap();

    Ok(Version { major, minor, patch })
}

pub struct Opts<'a> {
    pub repo_url: &'a str,
    pub username: &'a str,
    pub password: &'a str,
    pub destination: &'a Path
}

pub fn sync_repository(opts: Opts) -> Result<(),Error> {

    // Create the destination folder:
    std::fs::create_dir_all(&opts.destination).map_err(|e|
        err!("Could not create path '{}': {}", opts.destination.to_string_lossy(), e)
    )?;

    // Is the folder already a bare repo? It is if
    // it contains a file called HEAD.
    let mut dest_head = opts.destination.to_owned();
    dest_head.push("HEAD");
    let is_repo = dest_head.is_file();

    // Sync or clone depending on whether already a repo:
    let output = if is_repo {
        Command::new("sh")
            .arg("-c").arg(git_fetch_cmd())
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

    if !output.status.success() {
        Err(err!("Git command did not exit successfully: \n\n{}\n", String::from_utf8_lossy(&output.stderr)))
    } else {
        Ok(())
    }
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

fn git_fetch_cmd() -> String {
    String::from("git fetch origin '+*:*' --prune")
}
