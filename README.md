Imitation of Gobuster/ffuf in Rust.

### Todo

dir command:
[x] -u, --url string                        The target URL
[x] -w, --wordlist string       Path to the wordlist.
[x] -h, --help                              help for dir
[x] -b, --status-codes-blacklist string     Status codes that will be ignored (default "404")
[x] --exclude-length string             exclude the following content lengths (completely ignores the status). You can separate multiple lengths by comma and it also supports ranges like 203-206
[x] --headers               Custom headers. Use the format "Header1: Content1, Header2: Content2"
[ ] -c, --cookies string                  Cookies to use for the requests
[x] -m, --method string                   Use the following HTTP method (default "GET")
