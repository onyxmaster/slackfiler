use std::{
    borrow::Cow,
    io::{BufRead, BufReader, LineWriter, Read, Write},
};

pub(crate) trait LineProcessor {
    fn process<'a>(&self, text: &'a str) -> Result<Cow<'a, str>, Box<dyn std::error::Error>>;
}

pub(crate) struct ContentProcessor<'a> {
    pub(crate) line_processor: &'a dyn LineProcessor,
}

impl<'a> ContentProcessor<'a> {
    pub(crate) fn process<R: Read, W: Write>(
        &self,
        input: &mut R,
        output: &mut W,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut output = LineWriter::new(output);
        for line in BufReader::new(input).lines() {
            let line = line?;
            let line = self.line_processor.process(&line)?;
            output.write_all(line.as_bytes())?;
            output.write_all(b"\n")?;
        }
        Ok(())
    }
}
