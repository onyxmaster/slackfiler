use std::{
    io::{BufRead, BufReader, Error, Read},
    str::FromStr,
};

use regex::{Captures, Regex};

pub(crate) struct LinkProcessor {
    filter: Regex,
}

impl LinkProcessor {
    pub(crate) fn new() -> Self {
        LinkProcessor {
            filter: Regex::from_str(r#""(https?:\\/\\/.+?)""#).unwrap(),
        }
    }
}

impl LineProcessor for LinkProcessor {
    fn process(&self, text: &str) -> String {
        self.filter
            .replace_all(text, |captures: &Captures| {
                let url = captures[1].replace("\\/", "/");
                // Compute hash, check if it's already present, if not -- download, then replace link with local file
                format!("\"{}\"", url)
            })
            .to_string()
    }
}

pub(crate) trait LineProcessor {
    fn process(&self, text: &str) -> String;
}

pub(crate) struct FileProcessor<'a> {
    pub(crate) line_processor: &'a dyn LineProcessor,
}

impl<'a> FileProcessor<'a> {
    pub(crate) fn process<R: Read>(&self, file: &mut R) -> Result<(), Error> {
        let lines = BufReader::new(file).lines();
        for line in lines {
            let line = line?;
            let line = self.line_processor.process(&line);
            println!("{}", line);
        }
        Ok(())
    }
}
