use crate::error::Error;

pub trait Service {
    /// Which repositories do we want to back up?
    fn list_repositories(&self) -> Result<Vec<Repository>,Error>;
}

pub struct Repository {
    pub git_url: String,
    pub name: String
}