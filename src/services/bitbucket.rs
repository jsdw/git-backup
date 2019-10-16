use regex::Regex;
use lazy_static::lazy_static;
use crate::error::Error;
use super::service::{ Service, Repository };

pub struct Bitbucket {
    /// Which user are we backing up repositories for?
    owner: String,
    /// An access token
    token: String
}

impl Bitbucket {
    pub fn new(url: String, token: String) -> Option<Bitbucket> {
        lazy_static! {
            static ref HTTP_URL_RE: Regex = Regex::new("^(?:http(?:s)?://)?(?:www\\.)?bitbucket(?:\\.org)?/([^/]+)(?:/)?$").unwrap();
            static ref SSH_URL_RE: Regex = Regex::new("^(?:git@)?bitbucket(?:\\.org)?:([^/.]+)(?:/)?$").unwrap();
            static ref BASIC_SSH_RE: Regex = Regex::new("^([^@]+)@bitbucket(?:\\.org)?(?:/)?$").unwrap();
        }
        // In all of the regexs, first capture is owner
        let caps = HTTP_URL_RE.captures(&url)
            .or_else(|| SSH_URL_RE.captures(&url))
            .or_else(|| BASIC_SSH_RE.captures(&url))?;

        let owner = caps.get(1).unwrap().as_str().to_owned();

        Some(Bitbucket { owner, token })
    }
    #[cfg(test)]
    pub fn owner(&self) -> &str {
        &self.owner
    }
}

impl Service for Bitbucket {
    fn username(&self) -> String {
        self.owner.to_owned()
    }
    fn list_repositories(&self) -> Result<Vec<Repository>,Error> {

        let token = &self.token;
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
                .get(&url)
                .header("Authorization", format!("Basic {}", bearer_token))
                .send()
                .map_err(|e| err!("There was a problem talking to Bitbucket: {}", e))?;

            // Return an error if the response was not successful:
            let status = res.status();
            if !status.is_success() {
                return Err(match status.as_u16() {
                    401 => err!("Not authorized: is the app password that you provided for Bitbucket valid?"),
                    _ => err!("Error talking to Bitbucket: {} (code {})", status.canonical_reason().unwrap_or("Unknown"), status.as_str())
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
            ("http://www.bitbucket.org/jsdw", "jsdw"),
            ("http://www.bitbucket.org/jsdw/", "jsdw"),
            ("http://bitbucket.org/jsdw", "jsdw"),
            ("https://bitbucket.org/jsdw", "jsdw"),
            ("https://bitbucket/jsdw", "jsdw"),
            ("bitbucket.org/jsdw", "jsdw"),
            ("bitbucket.org/jsdw/", "jsdw"),
            ("bitbucket/jsdw", "jsdw"),
            ("git@bitbucket.org:jsdw", "jsdw"),
            ("git@bitbucket.org:jsdw/", "jsdw"),
            ("bitbucket.org:jsdw", "jsdw"),
            ("bitbucket.org:jsdw/", "jsdw"),
            ("bitbucket:jsdw", "jsdw"),
            ("jsdw@bitbucket.org", "jsdw"),
            ("jsdw@bitbucket", "jsdw"),
        ];
        for (url, owner) in urls {
            if let Some(gh) = Bitbucket::new(url.to_owned(), "token".to_owned()) {
                assert_eq!(gh.owner(), owner, "url {} expected owner {} but got {}", url, owner, gh.owner());
            } else {
                panic!("url {} was not parsed properly", url);
            }
        }
    }

}