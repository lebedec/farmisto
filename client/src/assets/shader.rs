use std::fs::File;
use std::io;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::time::Instant;

use ash::util::read_spv;
use ash::vk;
use log::{error, info};

use crate::assets::Asset;
use crate::engine::base::MyQueue;

pub type ShaderAsset = Asset<ShaderAssetData>;

pub struct ShaderAssetData {
    pub module: vk::ShaderModule,
}

impl ShaderAssetData {
    pub fn from_spirv_file<P: AsRef<Path>>(
        queue: &Arc<MyQueue>,
        path: P,
    ) -> Result<Self, ShaderAssetError> {
        let time = Instant::now();
        let mut file = File::open(path.as_ref()).map_err(ShaderAssetError::Io)?;
        let code = read_spv(&mut file).map_err(ShaderAssetError::Io)?;
        let info = vk::ShaderModuleCreateInfo::builder().code(&code);
        let module = unsafe {
            queue
                .device
                .create_shader_module(&info, None)
                .map_err(ShaderAssetError::Vulkan)?
        };
        info!(
            "Create shader module {:?} form {:?} in {:?}",
            module,
            path.as_ref().to_str(),
            time.elapsed(),
        );
        Ok(Self { module })
    }
}

#[derive(Debug)]
pub enum ShaderAssetError {
    Io(io::Error),
    Vulkan(vk::Result),
}

pub struct ShaderCompiler {
    program: String,
}

impl ShaderCompiler {
    pub fn new() -> Self {
        #[cfg(target_os = "macos")]
        let path = "tools/bin/glslangValidator";

        #[cfg(target_os = "windows")]
        let path = "tools/bin/glslangValidator.exe";

        Self {
            program: path.to_string(),
        }
    }

    pub fn version(&self) -> String {
        Command::new(&self.program)
            .arg("-v")
            .output()
            .map(|output| String::from_utf8(output.stdout).unwrap())
            .unwrap()
    }

    pub fn compile_file<P: AsRef<Path>>(&self, path: P) {
        let time = Instant::now();
        let input = path.as_ref().to_path_buf();
        let mut output = input.to_path_buf();
        output.set_extension(format!(
            "{}.spv",
            output.extension().unwrap().to_str().unwrap()
        ));
        let output = Command::new(&self.program)
            .arg("-V")
            .arg("--target-env")
            .arg("vulkan1.0")
            .arg("-o")
            .arg(&output)
            .arg(&input)
            .output()
            .map(|output| String::from_utf8(output.stdout).unwrap())
            .unwrap()
            .trim()
            .to_string();

        // successful output is input file name
        if output.len() != input.to_str().unwrap().len() {
            error!("Unable to compile file {:?}, {:?}", input, output);
        }

        info!("Compile shader {:?} in {:?}", input, time.elapsed());
    }
}
