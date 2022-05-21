use std::io::Write;
use futures::{StreamExt};
use crate::pack;


#[derive(Clone, serde::Serialize)]
pub struct ModDownloadedPayload {
    pub mod_id: String,
    pub pack_id: String
}

#[derive(Clone, serde::Serialize)]
pub struct ModDownloadErrorPayload {
    pub mod_id: String,
    pub pack_id: String,
    pub file_name: String,
    pub error: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct SavedModEntry {
    pub project_id: String,
    pub version_id: String,
    pub filenames: Vec<String>
}

impl ModrinthModData {
    pub async fn install_mod(&mut self, destination: &std::path::PathBuf, window: &tauri::Window, pack: &mut pack::Modpack) -> Result<SavedModEntry, String> {
        let client = reqwest::Client::new();
        let pack_id = pack.id.as_deref().unwrap();
        let mut filenames = Vec::<String>::new();
        println!("[debug] starting downloads of {} files for pack id {}", self.files.len(), pack_id);
        for file in &self.files {
            println!("[debug] downloading file {}, size {}", &file.filename, &file.size);
            let mut dest = std::fs::File::create(destination.join(&file.filename)).expect("Could not create file");
            match client
                .get(&file.url)
                .header("User-Agent", "mc-mod-manager/v1.0-alpha")
                .send()
                .await
            {
                Ok(response) => {
                    let mut stream = response.bytes_stream();
                    while let Some(result) = stream.next().await {
                        match result {
                            Ok(chunk) => {
                                if let Err(err) = dest.write(&chunk) {
                                    println!("[{}] Write Error: {}", &file.filename, err);
                                    break;
                                }
                            },
                            Err(err) => {
                                window.emit("download-mod", ModDownloadErrorPayload {
                                    mod_id: self.id.clone(),
                                    file_name: file.filename.clone(),
                                    pack_id: pack_id.to_string(),
                                    error: err.to_string()
                                }).ok();
                                println!("item {} failed:\n{}", &file.filename, &err); 
                                return Err(err.to_string())
                            }
                        }
                    }
                    println!("[debug] downloaded {}", &file.filename);
                    window.emit("download-mod", ModDownloadedPayload {
                        mod_id: self.id.clone(),
                        pack_id: pack_id.to_string()
                    }).ok();
                    filenames.push(file.filename.clone());
                    // TODO: insert into pack.mods
                },
                Err(err) => {
                    println!("Download failure for {}: {}", &file.filename, err);
                    return Err(err.to_string())
                }
            }
        }
        let save_entry = SavedModEntry {
            project_id: self.project_id.clone(),
            version_id: self.id.clone(),
            filenames
        };
        println!("[debug] downloads complete for pack {}", pack_id);
        Ok(save_entry)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ModrinthModData {
    pub id: String,
    pub project_id: String,
    pub author_id: String,
    pub featured: bool,
    pub name: String,
    pub version_number: String,
    pub changelog: String,
    pub changelog_url: Option<String>,
    pub date_published: String,
    pub downloads: i64,
    pub version_type: String,
    pub files: Vec<ModrinthFile>,
    // pub dependencies: Vec<Value>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ModrinthFile {
    pub hashes: ModrinthHashes,
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub size: i64,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ModrinthHashes {
    pub sha512: String,
    pub sha1: String,
}