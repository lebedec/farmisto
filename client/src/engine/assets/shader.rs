use log::{error, info};
use std::fs;
use std::path::Path;
use std::process::Command;

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
