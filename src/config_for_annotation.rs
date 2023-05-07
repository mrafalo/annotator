use serde::Deserialize;


#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub annotations_folder: String,
    pub images_folder: String,
}
