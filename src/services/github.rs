use regex::Regex;
use serde::{ Serialize };
use serde_json::json;
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
            static ref HTTP_URL_RE: Regex = Regex::new("^(?:http(?:s)?://)?(?:www\\.)?github(?:\\.com)?/([^/]+)(?:/([^/]+?))?(?:/|\\.git)?$").unwrap();
            static ref SSH_URL_RE: Regex = Regex::new("^(?:git@)?github(?:\\.com)?:([^/.]+)(?:/(.+?)(?:\\.git)?)?$").unwrap();
            static ref BASIC_SSH_RE: Regex = Regex::new("^([^@]+)@github(?:\\.com)?(?:(?:/|:)(.+?)(?:\\.git)?)?$").unwrap();
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
    fn username(&self) -> String {
        self.owner.to_owned()
    }
    fn list_repositories(&self) -> Result<Vec<Repository>,Error> {

        // If only one repository was asked for, just return it:
        if let Some(repo) = &self.repository {
            return Ok(vec![
                Repository {
                    git_url: format!("https://github.com/{user}/{repo}", user=self.owner, repo=repo),
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

        let mut cursor: Option<String> = None;
        let mut repos = vec![];

        // Make as many queries as we need to gather together all of the
        // repositories (we can only obtain 100 at a time):
        loop {

            // Our GraphQL Query and variables are serialized to JSON:
            let body = json!({
                "query": GRAPHQL_QUERY,
                "variables": {
                    "cursor": cursor,
                    "query": format!("user:{}", self.owner)
                }
            });

            // We make a request, sending our personal access token:
            let mut res = client
                .post("https://api.github.com/graphql")
                .header("Authorization", format!("bearer {}", token))
                .json(&body)
                .send()
                .map_err(|e| err!("Error talking to github: {}", e))?;

            if !res.status().is_success() {
                return Err(err!("Non-success response code from Github (code {})", res.status().as_str()))
            }

            // We convert our response back to a loosely typed JSON Value:
            let data: serde_json::Value = res
                .json()
                .map_err(|_| err!("Invalid JSON response from Github"))?;

            // Iterate the list of repositories we find, converting to our
            // well typed Repository struct on the way:
            let data = &data["data"]["search"];
            let this_repos = data["nodes"].as_array().unwrap_or(&empty);
            for repo in this_repos {

                let name = repo["name"].as_str().ok_or_else(|| err!("Invalid repo name"))?;
                let url = repo["url"].as_str().ok_or_else(|| err!("Invalid repo URL"))?;

                repos.push(Repository {
                    name: name.to_owned(),
                    git_url: url.to_owned()
                })

            }

            // Do we have an endCursor? If so, use it to try pulling the next
            // set of results. If not, we're done so break:
            cursor = data["pageInfo"]["endCursor"].as_str().map(|s| s.to_owned());
            if cursor.is_none() {
                break
            }

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