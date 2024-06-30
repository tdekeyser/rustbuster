use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use crate::{Error, Result};

pub struct Wordlist {
    filename: PathBuf,
    extensions: Vec<String>,
}

impl TryFrom<PathBuf> for Wordlist {
    type Error = Error;

    fn try_from(filename: PathBuf) -> Result<Self> {
        if !filename.exists() {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found").into());
        }

        Ok(Wordlist {
            filename,
            extensions: vec![String::default()],
        })
    }
}

impl Wordlist {
    pub fn set_extensions(&mut self, extensions: Vec<String>) {
        self.extensions = extensions.iter()
            .map(|ext| if ext.is_empty() {
                String::new()
            } else {
                format!(".{ext}")
            })
            .collect();
    }

    pub fn iter(&self) -> impl Iterator<Item=String> + '_ {
        let file = File::open(&self.filename).expect("exists");

        BufReader::new(file).lines()
            .map(|w| w.unwrap_or_default())
            .flat_map(move |w| self.extensions.iter()
                .map(|ext| format!("{w}{ext}"))
                .collect::<Vec<String>>())
    }

    pub fn len(&self) -> usize {
        self.iter().count()
    }
}


#[cfg(test)]
mod tests {
    use std::fs::{File, remove_file};
    use std::io::prelude::*;
    use std::path::PathBuf;

    use crate::Result;
    use crate::words::Wordlist;

    #[test]
    fn wordlist_can_iterate() -> Result<()> {
        let filename = "wordlist_can_iterate.txt";
        let mut file = File::create(filename)?;
        file.write_all(b"let\nme\nin")?;

        let wordlist = Wordlist::try_from(PathBuf::from(filename))?;

        assert_eq!(wordlist.len(), 3);

        let mut words = wordlist.iter();

        assert_eq!(words.next(), Some("let".to_string()));
        assert_eq!(words.next(), Some("me".to_string()));
        assert_eq!(words.next(), Some("in".to_string()));
        assert_eq!(words.next(), None);

        remove_file(filename).map_err(|e| e.into())
    }

    #[test]
    fn wordlist_expands_from_extensions() -> Result<()> {
        let filename = "wordlist_expands_from_extensions.txt";
        let mut file = File::create(filename)?;
        file.write_all(b"let\nme\nin")?;

        let mut wordlist = Wordlist::try_from(PathBuf::from(filename))?;
        wordlist.set_extensions(vec!["json".to_string(), "xml".to_string()]);

        assert_eq!(wordlist.len(), 6);

        let mut words = wordlist.iter();

        assert_eq!(words.next(), Some("let.json".to_string()));
        assert_eq!(words.next(), Some("let.xml".to_string()));
        assert_eq!(words.next(), Some("me.json".to_string()));
        assert_eq!(words.next(), Some("me.xml".to_string()));
        assert_eq!(words.next(), Some("in.json".to_string()));
        assert_eq!(words.next(), Some("in.xml".to_string()));
        assert_eq!(words.next(), None);

        remove_file(filename).map_err(|e| e.into())
    }
}
