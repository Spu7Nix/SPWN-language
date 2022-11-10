use std::{path::Path, fs::File, sync::MutexGuard, io::Write};
use std::convert::TryInto;

use octocrab::models::repos::Tag;
use octocrab::{Octocrab, params::repos::Commitish, repos::RepoHandler};

use crate::pckp::util::PckpMeta;

use super::util;

pub struct RemotePackage {
    meta: PckpMeta,
    owner: String,
    repo: String,
    version: Option<String>,
}

impl RemotePackage {
    pub fn get_commitish(&self) -> Commitish {
        if self.version.is_some() {
            Commitish(format!("tags/{}", self.meta.version))
        } else {
            Commitish("HEAD".to_string())
        }
    }

    pub async fn from_github_repo(owner: String, repo: String, version: Option<String>) -> Self {
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
            meta: serde_yaml::from_str(pckp_yaml).unwrap(),
            owner: owner.clone(),
            repo: repo.clone(),
            version,
        }
    }

    async fn ensure_tag_exist(&self) {
        let mut gh = Octocrab::builder().build().unwrap();
        let mut repo = gh
        .repos(self.owner.clone(), self.repo.clone())
        .list_tags()
        .send()
        .await;
        
        let mut page_number: u32 = 1;
        let mut found = false;
        while let Ok(page) = repo {
            repo = gh
                .repos(self.owner.clone(), self.repo.clone())
                .list_tags()
                .page(page_number)
                .send()
                .await;
            
            for t in page.items {
                if t.name == self.meta.version {
                    found = true;
                    break; // for good measure
                }
            }
        }

        if ! found {
            util::pckp_error(format!("Tag wasn't found for package {}/{}", self.owner, self.repo).as_str());
        }
    }

    // this should work
    pub async fn download(&self, reference: &Path) {
        // self.ensure_tag_exist().await;
        let gh = Octocrab::builder().build().unwrap();
        let repo = gh.repos(self.owner.clone(), self.repo.clone());
        let res = repo
            .download_tarball(self.get_commitish()) // TODO: check if tag is present else use HEAD
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap()
            .to_vec();
        
        let mut f = File::create(reference.join("ball")).unwrap();
        f.write_all(&res).unwrap();
    }
}