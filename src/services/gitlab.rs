use regex::Regex;
use lazy_static::lazy_static;
use crate::error::Error;
use super::service::{ Service, Repository };

pub struct GitLab {
    /// Which user are we backing up repositories for?
    owner: String,
    /// If we only want to backup a single repository,
    /// store it here:
    repository: Option<String>,
    /// An access token (needed if storing)
    token: Option<String>
}

impl GitLab {
    pub fn new(url: String, token: Option<String>) -> Option<GitLab> {
        lazy_static! {
            static ref HTTP_URL_RE: Regex = Regex::new("^(?:http(?:s)?://)?(?:www\\.)?gitlab(?:\\.org)?/([^/]+)(?:/([^/]+?))?(?:/|\\.git)?$").unwrap();
            static ref SSH_URL_RE: Regex = Regex::new("^(?:git@)?gitlab(?:\\.org)?:([^/.]+)(?:/(.+?)(?:\\.git)?)?$").unwrap();
            static ref BASIC_SSH_RE: Regex = Regex::new("^([^@]+)@gitlab(?:\\.org)?(?:(?:/|:)(.+?)(?:\\.git)?)?$").unwrap();
        }
        // In all of the regexs, first capture is owner, second is repo name
        let caps = HTTP_URL_RE.captures(&url)
            .or_else(|| SSH_URL_RE.captures(&url))
            .or_else(|| BASIC_SSH_RE.captures(&url))?;

        let owner = caps.get(1).unwrap().as_str().to_owned();
        let repository = caps.get(2).map(|c| c.as_str().to_owned());

        Some(GitLab {
            owner, repository, token
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

impl Service for GitLab {
    fn username(&self) -> String {
        self.owner.to_owned()
    }
    fn list_repositories(&self) -> Result<Vec<Repository>,Error> {

        // If only one repository was asked for, just return it:
        if let Some(repo) = &self.repository {
            return Ok(vec![
                Repository {
                    git_url: format!("https://gitlab.com/{user}/{repo}.git", user=self.owner, repo=repo),
                    name: repo.clone()
                }
            ])
        }

        // If no token was provided, we can't list every repo:
        let token = self.token.as_ref().ok_or_else(|| {
            err!("A token must be provided to obtain a list of your GitLab repositories")
        })?;

        let client = reqwest::Client::new();

        let url = format!("https://gitlab.com/api/v4/users/{user}/projects?simple=true&owned=true", user=self.owner);
        let empty = vec![];
        let mut res = client
            .get(&url)
            .header("Private-Token", token)
            .send()
            .map_err(|e| err!("There was a problem talking to GitLab: {}", e))?;

        // Return an error if the response was not successful:
        let status = res.status();
        if !status.is_success() {
            return Err(match status.as_u16() {
                401 => err!("Not authorized: is the app password that you provided for GitLab valid?"),
                _ => err!("Error talking to GitLab: {} (code {})", status.canonical_reason().unwrap_or("Unknown"), status.as_str())
            });
        }

        // We convert our response back to a loosely typed JSON Value:
        let data: serde_json::Value = res
            .json()
            .map_err(|_| err!("Invalid JSON response from GitLab"))?;

        let mut repos = vec![];
        let repo_values = data.as_array().unwrap_or(&empty);
        for repo in repo_values {

            let url = repo["http_url_to_repo"]
                .as_str()
                .ok_or_else(|| err!("Invalid clone URL"))?;

            let name = repo["path"]
                .as_str()
                .ok_or_else(|| err!("Invalid repo name"))?;

            // Push to our repo list:
            repos.push(Repository {
                name: name.to_owned(),
                git_url: url.to_owned()
            })
        }

        Ok(repos)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_valid_urls() {
        let urls = vec![
            ("http://www.gitlab.org/jsdw", "jsdw", None),
            ("http://www.gitlab.org/jsdw/git.backup", "jsdw", Some("git.backup")),
            ("http://www.gitlab.org/jsdw/git.backup.git", "jsdw", Some("git.backup")),
            ("http://gitlab.org/jsdw", "jsdw", None),
            ("http://gitlab.org/jsdw/git.backup", "jsdw", Some("git.backup")),
            ("http://gitlab.org/jsdw/git.backup.git", "jsdw", Some("git.backup")),
            ("https://gitlab.org/jsdw", "jsdw", None),
            ("https://gitlab/jsdw", "jsdw", None),
            ("https://gitlab.org/jsdw/git.backup", "jsdw", Some("git.backup")),
            ("https://gitlab.org/jsdw/git.backup.git", "jsdw", Some("git.backup")),
            ("gitlab.org/jsdw", "jsdw", None),
            ("gitlab/jsdw", "jsdw", None),
            ("gitlab.org/jsdw/git.backup", "jsdw", Some("git.backup")),
            ("gitlab.org/jsdw/git.backup.git", "jsdw", Some("git.backup")),
            ("git@gitlab.org:jsdw", "jsdw", None),
            ("git@gitlab.org:jsdw/git.backup", "jsdw", Some("git.backup")),
            ("git@gitlab.org:jsdw/git.backup.git", "jsdw", Some("git.backup")),
            ("gitlab.org:jsdw", "jsdw", None),
            ("gitlab:jsdw", "jsdw", None),
            ("gitlab.org:jsdw/git.backup", "jsdw", Some("git.backup")),
            ("gitlab.org:jsdw/git.backup.git", "jsdw", Some("git.backup")),
            ("jsdw@gitlab.org", "jsdw", None),
            ("jsdw@gitlab", "jsdw", None),
            ("jsdw@gitlab.org/git.backup", "jsdw", Some("git.backup")),
            ("jsdw@gitlab/git.backup", "jsdw", Some("git.backup")),
            ("jsdw@gitlab.org/git.backup.git", "jsdw", Some("git.backup")),
            ("jsdw@gitlab.org:git.backup", "jsdw", Some("git.backup")),
            ("jsdw@gitlab.org:git.backup.git", "jsdw", Some("git.backup")),
        ];
        for (url, owner, repo) in urls {
            if let Some(gh) = GitLab::new(url.to_owned(), None) {
                assert_eq!(gh.owner(), owner, "url {} expected owner {} but got {}", url, owner, gh.owner());
                assert_eq!(gh.repo(), repo, "url {} expected repo {:?} but got {:?}", url, repo, gh.repo());
            } else {
                panic!("url {} was not parsed properly", url);
            }
        }
    }

}