use std::{path::PathBuf, fs::File, sync::MutexGuard, io::Write};
use std::convert::TryInto;

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
            .repos(&owner, &repo)
            .raw_file(Commitish("HEAD".to_string()), "/pckp.yaml")
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        
        Self {
            meta: serde_yaml::from_str(&pckp_yaml).unwrap(),
            owner: owner.clone(),
            repo: repo.clone()
        }
    }

    // this should work
    pub async fn download(&self, target: PathBuf) {
        let mut gh = Octocrab::builder().build().unwrap();
        let repo = gh.repos(self.owner.clone(), self.repo.clone());
        let res = repo
            .download_tarball(Commitish(format!("tags/{}", self.meta.version)))
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap()
            .into_iter()
            .collect::<Vec<u8>>();
        
        let mut f = File::create(target).unwrap();
        f.write_all(&res).unwrap();
    }
}