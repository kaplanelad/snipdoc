use std::{fs::File, io, io::Read, path::Path};

pub struct RFile {
    pub content: String,
}

impl RFile {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        Ok(Self {
            content: String::from_utf8_lossy(&buffer).to_string(),
        })
    }
}
