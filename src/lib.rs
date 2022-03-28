mod fs_tools;
mod processing;

use std::{error::Error, fs::File};

use fs_tools::FileIterator;
use processing::{FileProcessor, LinkProcessor};

pub fn run() -> Result<(), Box<dyn Error>> {
    let processor = FileProcessor {
        line_processor: &LinkProcessor::new(),
    };
    let list = FileIterator::new("data")?;
    for path in list {
        let mut file = File::open(path?)?;
        processor.process(&mut file)?;
    }
    Ok(())
}
