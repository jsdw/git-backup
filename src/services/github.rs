use regex::Regex;
use lazy_static::lazy_static;
use crate::error::Error;
use super::service::{ Service, Repository };

pub struct Github {
    /// Which user are we backing up repositories for?
    owner: String,
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
            static ref HTTP_URL_RE: Regex = Regex::new("^(?:http(?:s)?://)?(?:www\\.)?github.com/([^/]+)(?:/([^/]+?))?(?:/|\\.git)?$").unwrap();
            static ref SSH_URL_RE: Regex = Regex::new("^(?:git@)?github.com:([^/.]+)(?:/(.+?)(?:\\.git)?)?$").unwrap();
            static ref BASIC_SSH_RE: Regex = Regex::new("^([^@]+)@github.com(?:(?:/|:)(.+?)(?:\\.git)?)?$").unwrap();
        }
        // In all of the regexs, first capture is owner, second is repo name
        let caps = HTTP_URL_RE.captures(&url)
            .or_else(|| SSH_URL_RE.captures(&url))
            .or_else(|| BASIC_SSH_RE.captures(&url))?;

        let owner = caps.get(1).unwrap().as_str().to_owned();
        let repository = caps.get(2).map(|c| c.as_str().to_owned());

        Some(Github {
            owner, repository, token, public
        })
    }
    #[cfg(test)]
    pub fn owner(&self) -> &str {
        &self.owner
    }
    #[cfg(test)]
    pub fn repo(&self) -> Option<&str> {
        self.repository.as_ref().map(|s| &**s)
    }
}

impl Service for Github {
    fn list_repositories(&self) -> Result<Vec<Repository>,Error> {
        unimplemented!()
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_valid_urls() {
        let urls = vec![
            ("http://www.github.com/jsdw", "jsdw", None),
            ("http://www.github.com/jsdw/git.backup", "jsdw", Some("git.backup")),
            ("http://www.github.com/jsdw/git.backup.git", "jsdw", Some("git.backup")),
            ("http://github.com/jsdw", "jsdw", None),
            ("http://github.com/jsdw/git.backup", "jsdw", Some("git.backup")),
            ("http://github.com/jsdw/git.backup.git", "jsdw", Some("git.backup")),
            ("https://github.com/jsdw", "jsdw", None),
            ("https://github.com/jsdw/git.backup", "jsdw", Some("git.backup")),
            ("https://github.com/jsdw/git.backup.git", "jsdw", Some("git.backup")),
            ("github.com/jsdw", "jsdw", None),
            ("github.com/jsdw/git.backup", "jsdw", Some("git.backup")),
            ("github.com/jsdw/git.backup.git", "jsdw", Some("git.backup")),
            ("git@github.com:jsdw", "jsdw", None),
            ("git@github.com:jsdw/git.backup", "jsdw", Some("git.backup")),
            ("git@github.com:jsdw/git.backup.git", "jsdw", Some("git.backup")),
            ("github.com:jsdw", "jsdw", None),
            ("github.com:jsdw/git.backup", "jsdw", Some("git.backup")),
            ("github.com:jsdw/git.backup.git", "jsdw", Some("git.backup")),
            ("jsdw@github.com", "jsdw", None),
            ("jsdw@github.com/git.backup", "jsdw", Some("git.backup")),
            ("jsdw@github.com/git.backup.git", "jsdw", Some("git.backup")),
            ("jsdw@github.com:git.backup", "jsdw", Some("git.backup")),
            ("jsdw@github.com:git.backup.git", "jsdw", Some("git.backup")),
        ];
        for (url, owner, repo) in urls {
            if let Some(gh) = Github::new(url.to_owned(), None, false) {
                assert_eq!(gh.owner(), owner, "url {} expected owner {} but got {}", url, owner, gh.owner());
                assert_eq!(gh.repo(), repo, "url {} expected repo {:?} but got {:?}", url, repo, gh.repo());
            } else {
                panic!("url {} was not parsed properly", url);
            }
        }
    }

}