# secrets-cli
A cli tool to convert secrets file to text

```
>>> tc-secrets -h
Secrets CLI

Usage: tc-secrets [OPTIONS] --filepath <FILEPATH>

Options:
  -f, --filepath <FILEPATH>  Location of the secrets file
  -t, --text <TEXT>          Encoded secrets text
  -p, --password <PASSWORD>  Password for encoding/decoding [default: secret]
  -c, --copy                 Whether to copy the output to clipboard
  -o, --overwrite            Whether to overwrite the file if it exists
  -h, --help                 Print help (see more with '--help')
  -V, --version              Print version
```