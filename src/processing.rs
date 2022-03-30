use std::{
    borrow::Cow,
    cell::RefCell,
    error::Error,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    str::FromStr,
};

use crypto::{digest::Digest, sha1::Sha1};
use regex::Regex;

use crate::content_processing::LineProcessor;

pub(crate) struct LinkProcessor {
    filter: Regex,
    digest: RefCell<Sha1>,
    client: reqwest::blocking::Client,
}

impl LinkProcessor {
    const CONTENT_DIR: &'static str = "content/";

    pub(crate) fn new() -> Result<Self, Box<dyn Error>> {
        Ok(LinkProcessor {
            filter: Regex::from_str(
                r#""(\w+)": ?"(https?:\\/\\/(?:\w+?\.)?(?:slack-files|slack-edge|files\.slack|gravatar)?\.com\\/.+?)""#,
            )?,
            digest: RefCell::new(Sha1::new()),
            client: reqwest::blocking::Client::new(),
        })
    }

    fn process_url<'a>(&self, url: &'a str) -> Result<Cow<'a, str>, Box<dyn Error>> {
        let url = url.replace("\\/", "/");
        let mut url = url::Url::from_str(&url)?;
        // Discard query string/fragment since we really don't need it for content
        url.set_fragment(None);
        let request_url = url.clone();
        url.set_query(None);
        let url = url;
        let filename = self.create_filename(&url);
        if let Some(mut file) = LinkProcessor::try_create_file(&filename)? {
            let content = self.client.get(request_url).send()?.bytes()?;
            file.write_all(&content)?;
        }
        let fragment = urlencoding::encode(url.as_str());
        let filename = filename.replace('/', "\\/");
        Ok(Cow::Owned(format!("\"{}#{}\"", filename, fragment)))
    }

    fn create_filename(&self, url: &url::Url) -> String {
        let mut digest = self.digest.borrow_mut();
        digest.reset();
        digest.input_str(url.as_str());
        let mut filename = digest.result_str();
        filename.insert(3, '/');
        filename.insert_str(0, LinkProcessor::CONTENT_DIR);
        if let Some(extension) = Path::new(url.path()).extension() {
            if extension.len() <= 4 {
                filename.push('.');
                filename.push_str(&extension.to_string_lossy());
            }
        }
        filename
    }

    fn try_create_file(filename: &str) -> Result<Option<fs::File>, std::io::Error> {
        fs::create_dir_all(Path::new(filename).parent().unwrap())?;
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(filename)
        {
            Ok(file) => Ok(Some(file)),
            Err(err) if err.raw_os_error() == Some(80) => Ok(None),
            Err(err) => Err(err),
        }
    }
}

impl LineProcessor for LinkProcessor {
    fn process<'a>(&self, text: &'a str) -> Result<Cow<'a, str>, Box<dyn Error>> {
        let mut captures_iter = self.filter.captures_iter(text);
        if let Some(mut captures) = captures_iter.next() {
            let mut result = String::new();
            let mut offset = 0usize;
            loop {
                let field = captures.get(1).unwrap().as_str();
                let capture = captures.get(2).unwrap();
                if let "url_private" | "permalink" | "permalink_public" | "from_url" | "thumb_64"
                | "thumb_80" | "thumb_160" | "thumb_360" | "thumb_480" | "thumb_720"
                | "thumb_800" | "image_24" | "image_32" | "image_48" | "image_72"
                | "image_192" | "image_512" | "image_1024" | "thumb_960" | "thumb_1024" = field
                {
                    result.push_str(&text[offset..capture.end()]);
                } else {
                    result.push_str(&text[offset..capture.start() - 1]);
                    let url = self.process_url(capture.as_str())?;
                    result.push_str(&url);
                }

                offset = capture.end() + 1;

                if let Some(next_captures) = captures_iter.next() {
                    captures = next_captures;
                } else {
                    break;
                }
            }
            result.push_str(&text[offset..]);
            if !result.is_empty() {
                return Ok(Cow::Owned(result));
            }
        }

        Ok(Cow::Borrowed(text))
    }
}
