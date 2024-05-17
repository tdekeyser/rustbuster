use std::fs;
use std::path::PathBuf;

use tokio::io;

pub struct Wordlist {
    words: Vec<String>,
}

impl TryFrom<PathBuf> for Wordlist {
    type Error = io::Error;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let words = fs::read_to_string(value)?
            .lines()
            .map(String::from)
            .collect();

        Ok(Wordlist { words })
    }
}

impl Wordlist {
    pub fn expand(self, extensions: Vec<String>) -> Wordlist {
        let words = self.words.iter()
            .flat_map(|word| extensions.iter()
                .map(|ext| format!("{}{}", word, ext))
                .collect::<Vec<String>>())
            .collect();

        Wordlist { words }
    }

    pub fn len(&self) -> usize {
        self.words.iter().count()
    }

    pub fn iter(self: &Self) -> impl Iterator<Item=&String> {
        self.words.iter()
    }
}


#[cfg(test)]
mod tests {
    use std::fs::{File, remove_file};
    use std::io;
    use std::io::prelude::*;
    use std::path::PathBuf;

    use crate::words::Wordlist;

    #[test]
    fn wordlist_reads_from_file() -> Result<(), io::Error> {
        let filename = "wordlist_reads_from_file.txt";
        let mut file = File::create(filename)?;
        file.write_all(b"let\nme\nin")?;

        let wordlist = Wordlist::try_from(PathBuf::from(filename))?;

        assert_eq!(wordlist.words, vec!["let", "me", "in"]);
        assert_eq!(wordlist.len(), 3);

        remove_file(filename)
    }

    #[test]
    fn wordlist_expands_from_extensions() -> Result<(), io::Error> {
        let filename = "wordlist_expands_from_extensions.txt";
        let mut file = File::create(filename)?;
        file.write_all(b"let\nme\nin")?;

        let mut wordlist = Wordlist::try_from(PathBuf::from(filename))?;
        wordlist = wordlist.expand(vec![".json".to_string(), ".xml".to_string()]);

        assert_eq!(wordlist.words,
                   vec![
                       "let.json", "let.xml",
                       "me.json", "me.xml",
                       "in.json", "in.xml",
                   ]);
        assert_eq!(wordlist.len(), 6);

        remove_file(filename)
    }
}
