# secrets-cli
A cli tool to convert secrets file to text

```
>>> tc-secrets -h
Secrets CLI

Usage: tc-secrets.exe [OPTIONS] --filepath <FILEPATH>

Options:
  -f, --filepath <FILEPATH>  Location of the secrets file
  -t, --text <TEXT>          Encoded secrets text
      --compare <COMPARE>    Another file to compare with the secrets file
  -p, --password <PASSWORD>  Password for encoding/decoding [default: secret]
  -c, --copy                 Whether to copy the output to clipboard
  -o, --overwrite            Whether to overwrite the file if it exists
  -h, --help                 Print help (see more with '--help')
  -V, --version              Print version
```

### Examples

Encode secrets file to text and copy to clipboard:
```
tc-secrets -f .env.local -c
```

Decode secrets text to file and overwrite if it exists:
```
tc-secrets -t "vWVYmMf0JgaNmzvVn13l4g==" -f .env.local -o
```

Compare secrets file with another file:
```
tc-secrets -f .env.local --compare .env
```