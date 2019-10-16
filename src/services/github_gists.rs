use regex::Regex;
use serde_json::json;
use lazy_static::lazy_static;
use std::collections::HashMap;
use crate::error::Error;
use super::service::{ Service, Repository };

pub struct GitHubGists {
    /// Which user are we backing up repositories for?
    owner: String,
    /// An access token
    token: String
}

impl GitHubGists {
    pub fn new(url: String, token: String) -> Option<GitHubGists> {
        lazy_static! {
            static ref HTTP_URL_RE: Regex = Regex::new("^(?:http(?:s)?://)?gist(?:s)?.github(?:\\.com)?/([^/]+)(?:/)?$").unwrap();
            static ref SSH_URL_RE: Regex = Regex::new("^(?:git@)?gist(?:s)?.github(?:\\.com)?:([^/.]+)(?:/)?$").unwrap();
            static ref BASIC_SSH_RE: Regex = Regex::new("^([^@]+)@gist(?:s)?.github(?:\\.com)?$").unwrap();
        }
        // Only capture the owner, don't try to capture the repo name,
        // because we'll want to map between ugly ID and nice name and so
        // we need the whole set of gists to do that sanely
        let caps = HTTP_URL_RE.captures(&url)
            .or_else(|| SSH_URL_RE.captures(&url))
            .or_else(|| BASIC_SSH_RE.captures(&url))?;

        let owner = caps.get(1).unwrap().as_str().to_owned();

        Some(GitHubGists { owner, token })
    }
    #[cfg(test)]
    pub fn owner(&self) -> &str {
        &self.owner
    }
}

impl Service for GitHubGists {
    fn username(&self) -> String {
        self.owner.to_owned()
    }
    fn list_repositories(&self) -> Result<Vec<Repository>,Error> {

        let token = &*self.token;
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
                    "user": self.owner
                }
            });

            // We make a request, sending our personal access token:
            let mut res = client
                .post("https://api.github.com/graphql")
                .header("Authorization", format!("bearer {}", token))
                .json(&body)
                .send()
                .map_err(|e| err!("There was a problem talking to github: {}", e))?;

            // Return an error if the response was not successful:
            let status = res.status();
            if !status.is_success() {
                return Err(match status.as_u16() {
                    401 => err!("Not authorized: is the personal access token that you provided for GitHub (Gists) valid?"),
                    _ => err!("Problem talking to github: {} (code {})", status.canonical_reason().unwrap_or("Unknown"), status.as_str())
                });
            }

            // We convert our response back to a loosely typed JSON Value:
            let data: serde_json::Value = res
                .json()
                .map_err(|_| err!("Invalid JSON response from GitHub (Gists)"))?;

            // Iterate the list of repositories we find, converting to our
            // well typed Repository struct on the way:
            let data = &data["data"]["user"]["gists"];
            let this_repos = data["nodes"].as_array().unwrap_or(&empty);
            for repo in this_repos {

               println!("Repo: {:?}", repo);
                let url = repo["url"].as_str().ok_or_else(|| err!("Invalid gist URL: {:?}", repo["url"]))?;
                let name = repo["files"][0]["name"].as_str().ok_or_else(|| err!("Invalid gist name"))?;

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

        // Names can be dupes, since they are based on the first file in the gist.
        // (this is how GitHub names gists, too). So, dedupe names, knowing that most
        // recent is always last (owing to ordering of createdAt) and so names should
        // be stable in the face of creating more dupe gists.
        let mut seen_name_counts: HashMap<String,usize> = HashMap::new();
        for repo in &mut repos {
            // Increment how many times the name is seen:
            let n = seen_name_counts.entry(repo.name.clone()).or_insert(0);
            *n += 1;
            // If name seen more than once, append number to it:
            if *n != 1 {
                repo.name = format!("{} {}", repo.name, n);
            }
        }

        Ok(repos)
    }
}

static GRAPHQL_QUERY: &str = "
    query ($user: String!, $cursor: String) {
        user(login: $user) {
            gists(first: 100, after: $cursor, privacy:ALL, orderBy: { field:CREATED_AT, direction:ASC }) {
                nodes {
                    url
                    createdAt
                    files(limit: 1) {
                        name
                    }
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
            ("http://gist.github.com/jsdw", "jsdw"),
            ("http://gists.github.com/jsdw", "jsdw"),
            ("https://gist.github.com/jsdw", "jsdw"),
            ("https://gists.github.com/jsdw", "jsdw"),
            ("https://gist.github/jsdw", "jsdw"),
            ("https://gists.github/jsdw", "jsdw"),
            ("gist.github.com/jsdw", "jsdw"),
            ("gist.github/jsdw", "jsdw"),
            ("git@gist.github.com:jsdw", "jsdw"),
            ("gist.github.com:jsdw", "jsdw"),
            ("gist.github:jsdw", "jsdw"),
            ("jsdw@gist.github.com", "jsdw"),
            ("jsdw@gist.github", "jsdw")
        ];
        for (url, owner) in urls {
            if let Some(gh) = GitHubGists::new(url.to_owned(), "token".to_owned()) {
                assert_eq!(gh.owner(), owner, "url {} expected owner {} but got {}", url, owner, gh.owner());
            } else {
                panic!("url {} was not parsed properly", url);
            }
        }
    }

}