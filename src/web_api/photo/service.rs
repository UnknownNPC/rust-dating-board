use std::{env, error::Error, ffi::OsStr, path::Path};

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
        profile_folder_name: &str,
    ) -> Result<PhotoOnFS, Box<dyn Error>> {
        let original_file_name = original_file.file_name.as_ref().unwrap();
        let original_file_extension = Path::new(&original_file_name)
            .extension()
            .and_then(OsStr::to_str)
            .unwrap_or("jpeg");

        let mut new_file_path = env::current_exe().unwrap();
        // remove binary name
        new_file_path.pop();
        // add global_folder + profile folder name
        new_file_path.push(all_photos_folder_name);
        new_file_path.push(profile_folder_name);

        if !new_file_path.exists() {
            println!(
                "[PhotoOnFS#save_photo_on_fs] creating folder for new file: {}",
                &new_file_path.to_str().unwrap()
            );
            fs::create_dir_all(&new_file_path)?;
        }

        let new_file_unique_id = Uuid::new_v4().to_string();
        let new_file_name = format!("{}.{1}", new_file_unique_id, original_file_extension);
        // add photo name with ext
        new_file_path.push(&new_file_name);
        println!(
            "[PhotoOnFS#save_photo_on_fs] forming path for new file: {}",
            &new_file_path.to_str().unwrap()
        );

        let from_file_path = original_file.file.path();
        println!(
            "[PhotoOnFS#save_photo_on_fs] copying data from file {} to file {1}",
            &from_file_path.to_str().unwrap(),
            &new_file_path.to_str().unwrap()
        );
        fs::copy(&from_file_path, &new_file_path)?;

        Ok(PhotoOnFS {
            name: new_file_name,
            // dirty usize 2 i64 converting
            size: original_file.size.to_string().parse::<i64>().unwrap(),
        })
    }
}
