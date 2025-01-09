use std::{
    env,
    ffi::OsStr,
    io,
    path::{Path, PathBuf},
};

use ab_glyph::FontRef;
use actix_multipart::form::tempfile::TempFile;
use image::{GenericImageView, ImageError};
use log::info;
use std::fs;
use uuid::Uuid;

use image::io::Reader as ImageReader;
use image::{DynamicImage, Rgba};
use imageproc::drawing::draw_text_mut;

pub static MAX_PROFILE_PHOTO_HEIGHT: &'static u32 = &550;
pub static MAX_PROFILE_PHOTO_WIDTH: &'static u32 = &360;

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
        fn image_error_to_io_error(err: &ImageError) -> io::Error {
            io::Error::new(io::ErrorKind::Other, format!("ImageError: {:?}", err))
        }

        fn image_scaling_post_processing(
            profile_photo_folder_path: &PathBuf,
        ) -> Result<(), io::Error> {
            let image_for_post_processing = image::open(&profile_photo_folder_path)
                .map_err(|err| image_error_to_io_error(&err))?;

            let (width, height) = image_for_post_processing.dimensions();

            if (width > *MAX_PROFILE_PHOTO_WIDTH) || height > *MAX_PROFILE_PHOTO_HEIGHT {
                info!("We need scaling: width: {}, height: {} ", width, height);

                let resized_img = image_for_post_processing.resize_to_fill(
                    *MAX_PROFILE_PHOTO_WIDTH,
                    *MAX_PROFILE_PHOTO_HEIGHT,
                    image::imageops::FilterType::Triangle,
                );

                resized_img
                    .save(&profile_photo_folder_path)
                    .map_err(|err| image_error_to_io_error(&err))
            } else {
                info!(
                    "We do not change scaling: width: {}, height: {}",
                    width, height
                );
                Ok(())
            }
        }

        fn add_watermark_post_processing(
            profile_photo_folder_path: &PathBuf,
        ) -> Result<(), io::Error> {
            info!(
                "Adding watermark for file: {}",
                &profile_photo_folder_path.to_str().unwrap()
            );
            let reader = ImageReader::open(profile_photo_folder_path)?;
            let mut img = reader
                .decode()
                .map_err(|err| image_error_to_io_error(&err))?;

            let color = Rgba([255, 255, 255, 255]);

            let text = "Anketa.VIP";

            let font = FontRef::try_from_slice(include_bytes!("microsoftsansserif.ttf")).unwrap();

            let (width, height) = img.dimensions();
            let position = (width - 100, height - 25);

            draw_text_mut(
                &mut img,
                color,
                position.0 as i32,
                position.1 as i32,
                20.0,
                &font,
                text,
            );

            let output_img: DynamicImage = img.into();
            output_img
                .save(profile_photo_folder_path)
                .map_err(|err| image_error_to_io_error(&err))
        }

        let original_file_name = original_file.file_name.as_ref().unwrap();
        let original_file_extension = Path::new(&original_file_name)
            .extension()
            .and_then(OsStr::to_str)
            .map(|f| f.to_lowercase())
            .unwrap_or(String::from("jpeg"));

        let mut profile_photo_folder_path =
            Self::get_path_2_profile_photos(all_photos_folder_name, profile_id);

        if !profile_photo_folder_path.exists() {
            info!(
                "Creating folder for new file: {}",
                &profile_photo_folder_path.to_str().unwrap()
            );
            fs::create_dir_all(&profile_photo_folder_path)?;
        }

        let new_file_unique_id = Uuid::new_v4().to_string();
        let new_file_name = format!("{}.{1}", new_file_unique_id, original_file_extension);
        // add photo name with ext
        profile_photo_folder_path.push(&new_file_name);
        info!(
            "Forming path for new file: {}",
            &profile_photo_folder_path.to_str().unwrap()
        );

        let from_file_path = original_file.file.path();
        info!(
            "Copying data from file {} to file {1}",
            &from_file_path.to_str().unwrap(),
            &profile_photo_folder_path.to_str().unwrap()
        );
        fs::copy(&from_file_path, &profile_photo_folder_path)?;

        image_scaling_post_processing(&profile_photo_folder_path)?;
        add_watermark_post_processing(&profile_photo_folder_path)?;

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
            info!(
                "Cant find file {}. Was it deleted manually? ",
                &profile_photo_old_path.to_str().unwrap()
            );
            return Ok(());
        }

        let mut profile_photo_new_path = profile_photo_folder_path.clone();
        profile_photo_new_path.push("delete_".to_owned() + photo_name);

        fs::rename(profile_photo_old_path, profile_photo_new_path)
    }

    pub fn delete_profile_from_fs(
        all_photos_folder_name: &str,
        profile_id: &Uuid,
    ) -> Result<(), io::Error> {
        let profile_photo_folder_path =
            Self::get_path_2_profile_photos(all_photos_folder_name, profile_id);
        if profile_photo_folder_path.exists() {
            let new_profile_photo_folder_path =
                profile_photo_folder_path.to_str().unwrap().to_owned() + "_delete";
            info!(
                "Deleting profile photo folder. From [{}] to [{}]",
                &profile_photo_folder_path.to_str().unwrap(),
                &new_profile_photo_folder_path
            );
            fs::rename(profile_photo_folder_path, new_profile_photo_folder_path)
        } else {
            info!("Profile folder doesn't exist. Skipping");
            Ok(())
        }
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
