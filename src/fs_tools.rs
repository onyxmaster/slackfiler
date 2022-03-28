use std::{
    fs::{self, ReadDir},
    io::Error,
    path::PathBuf,
};

pub(crate) struct FileIterator(Vec<ReadDir>);

impl FileIterator {
    pub(crate) fn new(path: &str) -> Result<Self, Error> {
        Ok(FileIterator(vec![fs::read_dir(path)?]))
    }
}

#[allow(clippy::large_enum_variant)]
enum FileListElement {
    Path(PathBuf),
    Dir(ReadDir),
}

impl From<ReadDir> for FileListElement {
    fn from(v: ReadDir) -> Self {
        Self::Dir(v)
    }
}

impl From<PathBuf> for FileListElement {
    fn from(v: PathBuf) -> Self {
        Self::Path(v)
    }
}

impl Iterator for FileIterator {
    type Item = Result<PathBuf, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let len = self.0.len();
            if len == 0 {
                break;
            }

            if let Some(dir_result) = self.0[len - 1].next() {
                match dir_result.and_then(|entry| {
                    let path = entry.path();
                    Ok(if entry.metadata()?.is_dir() {
                        FileListElement::from(fs::read_dir(&path)?)
                    } else {
                        FileListElement::from(path)
                    })
                }) {
                    Ok(FileListElement::Dir(dir)) => self.0.push(dir),
                    Ok(FileListElement::Path(path)) => return Some(Ok(path)),
                    Err(err) => return Some(Err(err)),
                }
            } else {
                self.0.pop();
            }
        }
        None
    }
}
