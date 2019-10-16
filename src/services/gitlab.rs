use regex::Regex;
use lazy_static::lazy_static;
use crate::error::Error;
use super::service::{ Service, Repository };

pub struct GitLab {
    /// Which user are we backing up repositories for?
    owner: String,
    /// An access token
    token: String
}

impl GitLab {
    pub fn new(url: String, token: String) -> Option<GitLab> {
        lazy_static! {
            static ref HTTP_URL_RE: Regex = Regex::new("^(?:http(?:s)?://)?(?:www\\.)?gitlab(?:\\.org)?/([^/]+)(?:/)?$").unwrap();
            static ref SSH_URL_RE: Regex = Regex::new("^(?:git@)?gitlab(?:\\.org)?:([^/.]+)(?:/)?$").unwrap();
            static ref BASIC_SSH_RE: Regex = Regex::new("^([^@]+)@gitlab(?:\\.org)?(?:/)?$").unwrap();
        }
        // In all of the regexs, first capture is owner
        let caps = HTTP_URL_RE.captures(&url)
            .or_else(|| SSH_URL_RE.captures(&url))
            .or_else(|| BASIC_SSH_RE.captures(&url))?;

        let owner = caps.get(1).unwrap().as_str().to_owned();

        Some(GitLab { owner, token })
    }
    #[cfg(test)]
    pub fn owner(&self) -> &str {
        &self.owner
    }
}

impl Service for GitLab {
    fn username(&self) -> String {
        self.owner.to_owned()
    }
    fn list_repositories(&self) -> Result<Vec<Repository>,Error> {

        let token = &self.token;
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
            ("http://www.gitlab.org/jsdw", "jsdw"),
            ("http://www.gitlab.org/jsdw/", "jsdw"),
            ("http://gitlab.org/jsdw", "jsdw"),
            ("https://gitlab.org/jsdw", "jsdw"),
            ("https://gitlab/jsdw", "jsdw"),
            ("gitlab.org/jsdw", "jsdw"),
            ("gitlab.org/jsdw/", "jsdw"),
            ("gitlab/jsdw", "jsdw"),
            ("git@gitlab.org:jsdw", "jsdw"),
            ("git@gitlab.org:jsdw/", "jsdw"),
            ("gitlab.org:jsdw", "jsdw"),
            ("gitlab.org:jsdw/", "jsdw"),
            ("gitlab:jsdw", "jsdw"),
            ("jsdw@gitlab.org", "jsdw"),
            ("jsdw@gitlab", "jsdw"),
        ];
        for (url, owner) in urls {
            if let Some(gh) = GitLab::new(url.to_owned(), "token".to_owned()) {
                assert_eq!(gh.owner(), owner, "url {} expected owner {} but got {}", url, owner, gh.owner());
            } else {
                panic!("url {} was not parsed properly", url);
            }
        }
    }

}