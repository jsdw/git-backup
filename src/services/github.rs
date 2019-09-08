use regex::Regex;
use lazy_static::lazy_static;
use crate::error::Error;
use super::service::{ Service, Repository };

pub struct Github {
    /// Which user are we backing up repositories for?
    user: String,
    /// If we only want to backup a single repository,
    /// store it here:
    repository: Option<String>,
    /// An access token (needed if storing)
    token: Option<String>,
    /// Public repos only?
    public: bool
}

impl Github {
    pub fn new(url: String, token: Option<String>, public: bool) -> Option<Github> {
        lazy_static! {
            static ref http_url_re: Regex = Regex::new("^(?:https://)?(?:www\\.)?github.com/([^/]+)(?:/([^/]+))?(?:/|\\.git)?$").unwrap();
            static ref ssh_url_re: Regex = Regex::new("^(?:git@)?github.com:([^/.]+)(?:/(.+?)(?:\\.git)?)?$").unwrap();
            static ref basic_ssh_re: Regex = Regex::new("^([^@]+)@github.com(:?/(.+?)(?:\\.git)?)?$").unwrap();
        }
        let caps = http_url_re.captures(&url)?;
        None
    }
}

impl Service for Github {
    fn list_repositories(&self) -> Result<Vec<Repository>,Error> {
        unimplemented!()
    }
}