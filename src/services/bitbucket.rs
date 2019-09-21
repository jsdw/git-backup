use regex::Regex;
use lazy_static::lazy_static;
use crate::error::Error;
use super::service::{ Service, Repository };

pub struct Bitbucket {
    /// Which user are we backing up repositories for?
    owner: String,
    /// If we only want to backup a single repository,
    /// store it here:
    repository: Option<String>,
    /// An access token (needed if storing)
    token: Option<String>
}

impl Bitbucket {
    pub fn new(url: String, token: Option<String>) -> Option<Bitbucket> {
        lazy_static! {
            static ref HTTP_URL_RE: Regex = Regex::new("^(?:http(?:s)?://)?(?:www\\.)?bitbucket(?:\\.org)?/([^/]+)(?:/([^/]+?))?(?:/|\\.git)?$").unwrap();
            static ref SSH_URL_RE: Regex = Regex::new("^(?:git@)?bitbucket(?:\\.org)?:([^/.]+)(?:/(.+?)(?:\\.git)?)?$").unwrap();
            static ref BASIC_SSH_RE: Regex = Regex::new("^([^@]+)@bitbucket(?:\\.org)?(?:(?:/|:)(.+?)(?:\\.git)?)?$").unwrap();
        }
        // In all of the regexs, first capture is owner, second is repo name
        let caps = HTTP_URL_RE.captures(&url)
            .or_else(|| SSH_URL_RE.captures(&url))
            .or_else(|| BASIC_SSH_RE.captures(&url))?;

        let owner = caps.get(1).unwrap().as_str().to_owned();
        let repository = caps.get(2).map(|c| c.as_str().to_owned());

        Some(Bitbucket {
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

impl Service for Bitbucket {
    fn username(&self) -> String {
        self.owner.to_owned()
    }
    fn list_repositories(&self) -> Result<Vec<Repository>,Error> {

        // If only one repository was asked for, just return it:
        if let Some(repo) = &self.repository {
            return Ok(vec![
                Repository {
                    git_url: format!("https://bitbucket.org/{user}/{repo}", user=self.owner, repo=repo),
                    name: repo.clone()
                }
            ])
        }

        // If no token was provided, we can't list every repo:
        let token = self.token.as_ref().ok_or_else(|| {
            err!("A token must be provided to obtain a list of your Github repositories")
        })?;

        let client = reqwest::Client::new();

        let mut maybe_url: Option<String> = Some(
            format!("https://api.bitbucket.org/2.0/repositories/{user}?fields=next,values.slug,values.scm,values.links.clone,values.is_private,values.owner.nickname&role=owner", user=self.owner)
        );
        let empty = vec![];
        let mut repos = vec![];

        let bearer_token = base64::encode(&format!("{user}:{token}", user=self.owner, token=token));

        // Make as many queries as we need to gather together all of the
        // repositories (we can only obtain 100 at a time):
        while let Some(url) = maybe_url {

            let mut res = client
                .post(&url)
                .header("Authorization", format!("bearer {}", bearer_token))
                .send()
                .map_err(|e| err!("There was a problem talking to bitbucket: {}", e))?;

            // Return an error if the response was not successful:
            let status = res.status();
            if !status.is_success() {
                return Err(match status.as_u16() {
                    401 => err!("Not authorized: is the app password that you provided for Bitbucket valid?"),
                    _ => err!("Error talking to bitbucket: {} (code {})", status.canonical_reason().unwrap_or("Unknown"), status.as_str())
                });
            }

            // We convert our response back to a loosely typed JSON Value:
            let data: serde_json::Value = res
                .json()
                .map_err(|_| err!("Invalid JSON response from Bitbucket"))?;

            // Prepare the next page:
            maybe_url = data["next"].as_str().map(|s| s.to_owned());

            let repo_values = data["values"].as_array().unwrap_or(&empty);
            for repo in repo_values {
                // Ignore non-git repos:
                if repo["scm"].as_str() != Some("git") {
                    continue
                }
                // Extract the name and URL from the JSON:
                let name = repo["slug"].as_str().ok_or_else(|| err!("Invalid repo name"))?;
                let clone = repo["links"]["clone"].as_array().ok_or_else(|| err!("Can't get repo URL"))?;
                let url = clone.into_iter()
                    .find(|val| val["name"].as_str() == Some("https"))
                    .ok_or_else(|| err!("Can't find HTTPS repo URL to clone from"))?
                    ["href"].as_str()
                    .ok_or_else(|| err!("Invalid clone URL"))?;
                // Push to our repo list:
                repos.push(Repository {
                    name: name.to_owned(),
                    git_url: url.to_owned()
                })
            }
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
            ("http://www.bitbucket.org/jsdw", "jsdw", None),
            ("http://www.bitbucket.org/jsdw/git.backup", "jsdw", Some("git.backup")),
            ("http://www.bitbucket.org/jsdw/git.backup.git", "jsdw", Some("git.backup")),
            ("http://bitbucket.org/jsdw", "jsdw", None),
            ("http://bitbucket.org/jsdw/git.backup", "jsdw", Some("git.backup")),
            ("http://bitbucket.org/jsdw/git.backup.git", "jsdw", Some("git.backup")),
            ("https://bitbucket.org/jsdw", "jsdw", None),
            ("https://bitbucket/jsdw", "jsdw", None),
            ("https://bitbucket.org/jsdw/git.backup", "jsdw", Some("git.backup")),
            ("https://bitbucket.org/jsdw/git.backup.git", "jsdw", Some("git.backup")),
            ("bitbucket.org/jsdw", "jsdw", None),
            ("bitbucket/jsdw", "jsdw", None),
            ("bitbucket.org/jsdw/git.backup", "jsdw", Some("git.backup")),
            ("bitbucket.org/jsdw/git.backup.git", "jsdw", Some("git.backup")),
            ("git@bitbucket.org:jsdw", "jsdw", None),
            ("git@bitbucket.org:jsdw/git.backup", "jsdw", Some("git.backup")),
            ("git@bitbucket.org:jsdw/git.backup.git", "jsdw", Some("git.backup")),
            ("bitbucket.org:jsdw", "jsdw", None),
            ("bitbucket:jsdw", "jsdw", None),
            ("bitbucket.org:jsdw/git.backup", "jsdw", Some("git.backup")),
            ("bitbucket.org:jsdw/git.backup.git", "jsdw", Some("git.backup")),
            ("jsdw@bitbucket.org", "jsdw", None),
            ("jsdw@bitbucket", "jsdw", None),
            ("jsdw@bitbucket.org/git.backup", "jsdw", Some("git.backup")),
            ("jsdw@bitbucket/git.backup", "jsdw", Some("git.backup")),
            ("jsdw@bitbucket.org/git.backup.git", "jsdw", Some("git.backup")),
            ("jsdw@bitbucket.org:git.backup", "jsdw", Some("git.backup")),
            ("jsdw@bitbucket.org:git.backup.git", "jsdw", Some("git.backup")),
        ];
        for (url, owner, repo) in urls {
            if let Some(gh) = Bitbucket::new(url.to_owned(), None) {
                assert_eq!(gh.owner(), owner, "url {} expected owner {} but got {}", url, owner, gh.owner());
                assert_eq!(gh.repo(), repo, "url {} expected repo {:?} but got {:?}", url, repo, gh.repo());
            } else {
                panic!("url {} was not parsed properly", url);
            }
        }
    }

}