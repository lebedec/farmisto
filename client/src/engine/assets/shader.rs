use crate::engine::base::Queue;
use ash::util::read_spv;
use ash::vk;
use ash::vk::ShaderModule;
use log::error;
use std::cell::RefCell;
use std::fs::File;
use std::io;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

#[derive(Clone)]
pub struct ShaderAsset {
    data: Arc<RefCell<ShaderAssetData>>,
}

impl ShaderAsset {
    #[inline]
    pub fn module(&self) -> ShaderModule {
        self.data.borrow().module
    }

    #[inline]
    pub fn update(&self, data: ShaderAssetData) {
        let mut this = self.data.borrow_mut();
        *this = data;
    }

    pub fn from_data(data: Arc<RefCell<ShaderAssetData>>) -> Self {
        Self { data }
    }
}

#[derive(Clone)]
pub struct ShaderAssetData {
    module: ShaderModule,
}

impl ShaderAssetData {
    pub fn from_spirv_file<P: AsRef<Path>>(
        queue: &Arc<Queue>,
        path: P,
    ) -> Result<Self, ShaderAssetError> {
        let mut file = File::open(path).map_err(ShaderAssetError::Io)?;
        let code = read_spv(&mut file).map_err(ShaderAssetError::Io)?;
        let info = vk::ShaderModuleCreateInfo::builder().code(&code);

        let module = unsafe {
            queue
                .device
                .create_shader_module(&info, None)
                .map_err(ShaderAssetError::Vulkan)?
        };

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
    }
}
