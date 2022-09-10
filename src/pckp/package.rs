use std::path::PathBuf;

use octocrab::{Octocrab, params::repos::Commitish, repos::RepoHandler};
use crate::pckp::util::PckpMeta;

pub struct RemotePackage {
    meta: PckpMeta,
    owner: String,
    repo: String,
}

impl RemotePackage {
    pub async fn from_github_repo(owner: String, repo: String) -> Self {
        let mut gh = Octocrab::builder().build().unwrap();
        let pckp_yaml = &gh
            .repos(owner, repo)
            .raw_file(Commitish("HEAD".to_string()), "/pckp.yaml")
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        
        Self {
            meta: serde_yaml::from_str(&pckp_yaml).unwrap(),
            owner,
            repo
        }
    }

    pub async fn download(&self, target: PathBuf) {
        let res = self.repo.download_tarball(Commitish(format!("tags/{}", self.meta.version))).await.unwrap().bytes().unwrap();
    }
}