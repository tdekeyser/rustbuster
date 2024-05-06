# Rustbuster

Imitation of Gobuster/ffuf in Rust

```
Usage: rustbuster [OPTIONS] --url <URL> --wordlist <WORDLIST>

Options:
  -u, --url <URL>
          The target URL
  -w, --wordlist <WORDLIST>
          Path to the wordlist
  -m, --method <METHOD>
          Use the following HTTP method (default "GET") [default: GET]
  -b, --blacklist-status-codes <BLACKLIST_STATUS_CODES>
          Status code that will be ignored, e.g. 404,500 [default: 404]
      --exclude-length <EXCLUDE_LENGTH>
          Content lengths that will be ignored, e.g. 20,300, or a range, e.g. 20-300 [default: Empty]
      --headers <HEADERS>
          Custom headers; use the format "Header1: Content1, Header2: Content2"
  -h, --help
          Print help
  -V, --version
          Print version
```
