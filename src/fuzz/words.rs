use std::path::PathBuf;

struct Wordlist {
    words: Vec<String>,
}

impl TryFrom<PathBuf> for Wordlist {
    type Error = ();

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl Wordlist {
    fn expand(self, extensions: Vec<String>) -> Wordlist {
        todo!()
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn wordlist_reads_from_file() {
        todo!()
    }

    #[test]
    fn wordlist_expands_from_extensions() {
        todo!()
    }
}
