# Rustbuster

Imitation of Gobuster/ffuf in Rust

```
Usage: rustbuster [OPTIONS] --url <URL> --wordlist <WORDLIST>

Options:
  -u, --url <URL>
          The target URL
  -w, --wordlist <WORDLIST>
          Path to the wordlist
  -x, --extensions <EXTENSIONS>
          File extensions to search for, e.g. json,xml [default: ]
  -m, --method <METHOD>
          Use the following HTTP method [default: GET]
  -H, --headers <HEADERS>
          Custom headers; use the format "Header1: Content1, Header2: Content2"
      --filter-status-codes <FILTER_STATUS_CODES>
          Status code that will be ignored, e.g. 404,500 [default: 404]
      --filter-content-length <FILTER_CONTENT_LENGTH>
          Content lengths that will be ignored, e.g. 20,300, or a range, e.g. 20-300 [default: Empty]
      --filter-body <FILTER_BODY>
          Ignore if text appears in the response body [default: Empty]
  -h, --help
          Print help
  -V, --version
          Print version
```
