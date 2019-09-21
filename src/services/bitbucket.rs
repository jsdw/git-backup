use regex::Regex;
use serde_json::json;
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
        let empty = Vec::new();

        let mut url: String = format!("https://bitbucket.org/api/2.0/repositories/{user}", user=self.owner);
        let mut repos = vec![];

        let bearerToken = base64::encode(&format!("{user}:{token}", user=self.owner, token=token));

        // Make as many queries as we need to gather together all of the
        // repositories (we can only obtain 100 at a time):
        loop {

            let mut res = client
                .post(&url)
                .header("Authorization", format!("bearer {}", bearerToken))
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



            // // We make a request, sending our personal access token:
            // let mut res = client
            //     .post("https://api.github.com/graphql")
            //     .header("Authorization", format!("bearer {}", token))
            //     .json(&body)
            //     .send()
            //     .map_err(|e| err!(" talking to github: {}", e))?;

            // // Return an error if the response was not successful:
            // let status = res.status();
            // if !status.is_success() {
            //     return Err(match status.as_u16() {
            //         401 => err!("Not authorized: is the personal access token that you provided valid?"),
            //         _ => err!("Error talking to github: {} (code {})", status.canonical_reason().unwrap_or("Unknown"), status.as_str())
            //     });
            // }

            // // We convert our response back to a loosely typed JSON Value:
            // let data: serde_json::Value = res
            //     .json()
            //     .map_err(|_| err!("Invalid JSON response from Github"))?;

            // // Iterate the list of repositories we find, converting to our
            // // well typed Repository struct on the way:
            // let data = &data["data"]["search"];
            // let this_repos = data["nodes"].as_array().unwrap_or(&empty);
            // for repo in this_repos {

            //     let name = repo["name"].as_str().ok_or_else(|| err!("Invalid repo name"))?;
            //     let url = repo["url"].as_str().ok_or_else(|| err!("Invalid repo URL"))?;

            //     repos.push(Repository {
            //         name: name.to_owned(),
            //         git_url: url.to_owned()
            //     })

            // }

            // // Do we have an endCursor? If so, use it to try pulling the next
            // // set of results. If not, we're done so break:
            // cursor = data["pageInfo"]["endCursor"].as_str().map(|s| s.to_owned());
            // if cursor.is_none() {
            //     break
            // }

        }

        Ok(repos)
    }
}

static GRAPHQL_QUERY: &str = "
    query($cursor: String, $query: String!) {
        search(query:$query, type:REPOSITORY, first:100, after:$cursor) {
            pageInfo {
                endCursor
            }
            nodes {
                ... on Repository {
                    url,
                    name
                }
            }
        }
    }
";

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
            ("https://github/jsdw", "jsdw", None),
            ("https://github.com/jsdw/git.backup", "jsdw", Some("git.backup")),
            ("https://github.com/jsdw/git.backup.git", "jsdw", Some("git.backup")),
            ("github.com/jsdw", "jsdw", None),
            ("github/jsdw", "jsdw", None),
            ("github.com/jsdw/git.backup", "jsdw", Some("git.backup")),
            ("github.com/jsdw/git.backup.git", "jsdw", Some("git.backup")),
            ("git@github.com:jsdw", "jsdw", None),
            ("git@github.com:jsdw/git.backup", "jsdw", Some("git.backup")),
            ("git@github.com:jsdw/git.backup.git", "jsdw", Some("git.backup")),
            ("github.com:jsdw", "jsdw", None),
            ("github:jsdw", "jsdw", None),
            ("github.com:jsdw/git.backup", "jsdw", Some("git.backup")),
            ("github.com:jsdw/git.backup.git", "jsdw", Some("git.backup")),
            ("jsdw@github.com", "jsdw", None),
            ("jsdw@github", "jsdw", None),
            ("jsdw@github.com/git.backup", "jsdw", Some("git.backup")),
            ("jsdw@github/git.backup", "jsdw", Some("git.backup")),
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