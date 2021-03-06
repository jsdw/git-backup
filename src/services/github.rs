use regex::Regex;
use serde_json::json;
use lazy_static::lazy_static;
use crate::error::Error;
use super::service::{ Service, Repository };

pub struct GitHub {
    /// Which user are we backing up repositories for?
    owner: String,
    /// An access token
    token: String
}

impl GitHub {
    pub fn new(url: String, token: String) -> Option<GitHub> {
        lazy_static! {
            static ref HTTP_URL_RE: Regex = Regex::new("^(?:http(?:s)?://)?(?:www\\.)?github(?:\\.com)?/([^/]+)(?:/)?$").unwrap();
            static ref SSH_URL_RE: Regex = Regex::new("^(?:git@)?github(?:\\.com)?:([^/.]+)(?:/)?$").unwrap();
            static ref BASIC_SSH_RE: Regex = Regex::new("^([^@]+)@github(?:\\.com)?(?:/)?$").unwrap();
        }

        // In all of the regexs, first capture is owner, second is repo name
        let caps = HTTP_URL_RE.captures(&url)
            .or_else(|| SSH_URL_RE.captures(&url))
            .or_else(|| BASIC_SSH_RE.captures(&url))?;

        let owner = caps.get(1).unwrap().as_str().to_owned();

        Some(GitHub { owner, token })
    }
    #[cfg(test)]
    pub fn owner(&self) -> &str {
        &self.owner
    }
}

impl Service for GitHub {
    fn username(&self) -> String {
        self.owner.to_owned()
    }
    fn list_repositories(&self) -> Result<Vec<Repository>,Error> {

        let token = &self.token;
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
                    401 => err!("Not authorized: is the personal access token that you provided for GitHub valid?"),
                    _ => err!("Problem talking to github: {} (code {})", status.canonical_reason().unwrap_or("Unknown"), status.as_str())
                });
            }

            // We convert our response back to a loosely typed JSON Value:
            let data: serde_json::Value = res
                .json()
                .map_err(|_| err!("Invalid JSON response from GitHub"))?;

            // Iterate the list of repositories we find, converting to our
            // well typed Repository struct on the way:
            let data = &data["data"]["user"]["repositories"];
            let this_repos = data["nodes"].as_array().unwrap_or(&empty);
            for repo in this_repos {
                let name = repo["name"].as_str().ok_or_else(|| err!("Invalid repo name: {:?}", repo["name"]))?;
                let url = repo["url"].as_str().ok_or_else(|| err!("Invalid repo URL: {:?}", repo["url"]))?;

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
    query($user:String!,$cursor:String) {
        user(login:$user) {
            repositories(first:100,after:$cursor,ownerAffiliations:OWNER,isFork:false) {
                pageInfo {
                    endCursor
                }
                nodes {
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
            ("http://www.github.com/jsdw", "jsdw"),
            ("http://www.github.com/jsdw/", "jsdw"),
            ("http://github.com/jsdw", "jsdw"),
            ("https://github.com/jsdw", "jsdw"),
            ("https://github/jsdw", "jsdw"),
            ("github.com/jsdw", "jsdw"),
            ("github.com/jsdw/", "jsdw"),
            ("github/jsdw", "jsdw"),
            ("git@github.com:jsdw", "jsdw"),
            ("git@github.com:jsdw/", "jsdw"),
            ("github.com:jsdw", "jsdw"),
            ("github.com:jsdw/", "jsdw"),
            ("github:jsdw", "jsdw"),
            ("jsdw@github.com", "jsdw"),
            ("jsdw@github", "jsdw"),
        ];
        for (url, owner) in urls {
            if let Some(gh) = GitHub::new(url.to_owned(), "token".to_owned()) {
                assert_eq!(gh.owner(), owner, "url {} expected owner {} but got {}", url, owner, gh.owner());
            } else {
                panic!("url {} was not parsed properly", url);
            }
        }
    }

}