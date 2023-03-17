use std::{
    env,
    ffi::OsStr,
    io,
    path::{Path, PathBuf},
};

use actix_multipart::form::tempfile::TempFile;
use std::fs;
use uuid::Uuid;

pub struct Service;

#[derive(Debug)]
pub struct PhotoOnFS {
    pub name: String,
    pub size: i64,
}

impl<'a> Service {
    pub fn save_photo_on_fs(
        original_file: &TempFile,
        all_photos_folder_name: &str,
        profile_id: &Uuid,
    ) -> Result<PhotoOnFS, io::Error> {
        let original_file_name = original_file.file_name.as_ref().unwrap();
        let original_file_extension = Path::new(&original_file_name)
            .extension()
            .and_then(OsStr::to_str)
            .map(|f| f.to_lowercase())
            .unwrap_or(String::from("jpeg"));

        let mut profile_photo_folder_path =
            Self::get_path_2_profile_photos(all_photos_folder_name, profile_id);

        if !profile_photo_folder_path.exists() {
            println!(
                "[PhotoOnFS#save_photo_on_fs] creating folder for new file: {}",
                &profile_photo_folder_path.to_str().unwrap()
            );
            fs::create_dir_all(&profile_photo_folder_path)?;
        }

        let new_file_unique_id = Uuid::new_v4().to_string();
        let new_file_name = format!("{}.{1}", new_file_unique_id, original_file_extension);
        // add photo name with ext
        profile_photo_folder_path.push(&new_file_name);
        println!(
            "[PhotoOnFS#save_photo_on_fs] forming path for new file: {}",
            &profile_photo_folder_path.to_str().unwrap()
        );

        let from_file_path = original_file.file.path();
        println!(
            "[PhotoOnFS#save_photo_on_fs] copying data from file {} to file {1}",
            &from_file_path.to_str().unwrap(),
            &profile_photo_folder_path.to_str().unwrap()
        );
        fs::copy(&from_file_path, &profile_photo_folder_path)?;

        Ok(PhotoOnFS {
            name: new_file_name,
            // dirty usize 2 i64 converting
            size: original_file.size.to_string().parse::<i64>().unwrap(),
        })
    }

    pub fn delete_photo_from_fs(
        all_photos_folder_name: &str,
        profile_id: &Uuid,
        photo_name: &str,
    ) -> Result<(), io::Error> {
        let profile_photo_folder_path =
            Self::get_path_2_profile_photos(all_photos_folder_name, profile_id);

        // move original photo
        let mut profile_photo_old_path = profile_photo_folder_path.clone();
        profile_photo_old_path.push(photo_name);

        if !profile_photo_old_path.exists() {
            println!(
                "[PhotoOnFS#delete_photo_from_fs] cant find file {}. Was it deleted manually? ",
                &profile_photo_old_path.to_str().unwrap()
            );
            return Ok(());
        }

        let mut profile_photo_new_path = profile_photo_folder_path.clone();
        profile_photo_new_path.push("delete_".to_owned() + photo_name);

        fs::rename(profile_photo_old_path, profile_photo_new_path)
    }

    fn get_path_2_profile_photos(all_photos_folder_name: &str, profile_id: &Uuid) -> PathBuf {
        let mut new_file_path = env::current_exe().unwrap();
        // remove binary name
        new_file_path.pop();
        // add global_folder + profile folder name
        new_file_path.push(all_photos_folder_name);
        new_file_path.push(profile_id.to_string());

        new_file_path
    }
}
