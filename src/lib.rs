mod fs_tools;
mod processing;

use std::{
    error::Error,
    ffi::OsStr,
    fs::{File, OpenOptions},
};

use fs_tools::FileIterator;
use processing::{FileProcessor, LinkProcessor};

pub fn run() -> Result<(), Box<dyn Error>> {
    let processor = FileProcessor {
        line_processor: &LinkProcessor::new()?,
    };
    let json_extension = OsStr::new("json");
    let list = FileIterator::new("data")?;
    for path in list {
        let input_path = path?;
        if input_path.extension() == Some(json_extension) {
            println!("{}", input_path.to_string_lossy());
            let mut output_path = input_path.as_os_str().to_os_string();
            output_path.push(".");
            output_path.push("downloaded");
            let output_path = output_path;
            let mut input = File::open(input_path)?;
            let mut output = OpenOptions::new()
                .write(true)
                .create(true)
                .open(&output_path)?;
            processor.process(&mut input, &mut output)?;
        }
    }
    Ok(())
}
