use std::{path::PathBuf, pin::Pin, sync::Mutex};

use crate::{
    options::{ContextOptions, ModelOptions},
    Result,
};

#[cfg(feature = "llama")]
pub mod llama;

pub trait Context {
    fn eval_str(&mut self, prompt: &str, add_bos: bool) -> Result<()>;
    fn eval_image(&mut self, image: Vec<u8>) -> Result<()>;
    fn predict(&mut self, max_len: usize) -> Result<String>;
    fn predict_with_callback(
        &mut self,
        token_callback: Box<dyn Fn(String) -> bool + Send + 'static>,
        max_len: usize,
    ) -> Result<()>;
}

pub trait Model {
    fn with_mmproj(&mut self, mmproj: PathBuf) -> Result<()>;
    fn new_context(&self, opions: ContextOptions) -> Result<Pin<Box<Mutex<dyn Context + '_>>>>;
}

pub fn init(model: impl Into<PathBuf>, options: ModelOptions) -> Result<impl Model> {
    #[cfg(feature = "llama")]
    Ok(llama::Llama::new(model, options)?)
}
