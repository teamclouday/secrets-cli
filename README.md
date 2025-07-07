# secrets-cli
A CLI tool for synchronizing .env files with AWS Secrets Manager

### Prerequisites

- AWS CLI installed and configured

### Commands

```
>>> tc-secrets -h
A CLI tool for synchronizing .env secret files with AWS Secrets Manager

Usage: tc-secrets.exe <COMMAND>

Commands:
  auth    Authenticate with AWS Secrets Manager
  diff    Display differences between local and remote secret files
  update  Increase the version of the local secret file
  sync    Synchronize local secret file with AWS Secrets Manager
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

```
>>> tc-secrets auth -h
Authenticate with AWS Secrets Manager

Usage: tc-secrets.exe auth

Options:
  -h, --help  Print help
```

```
>>> tc-secrets diff -h
Display differences between local and remote secret files

Usage: tc-secrets.exe diff [OPTIONS] --filepath <FILEPATH>

Options:
  -f, --filepath <FILEPATH>  Path to the local secret file
  -p, --password <PASSWORD>  Optional password for decrypting the secret file [default: secret]
  -h, --help                 Print help
```

```
>>> tc-secrets update -h
Increase the version of the local secret file

Usage: tc-secrets.exe update --filepath <FILEPATH>

Options:
  -f, --filepath <FILEPATH>  Path to the local secret file
  -h, --help                 Print help
```

```
>>> tc-secrets sync -h
Synchronize local secret file with AWS Secrets Manager

Usage: tc-secrets.exe sync [OPTIONS] --filepath <FILEPATH>

Options:
  -f, --filepath <FILEPATH>  Path to the local secret file
  -p, --password <PASSWORD>  Optional password for decrypting the secret file [default: secret]
  -h, --help                 Print help
```

### Usage

1. First authenticate with AWS Secrets Manager:
   ```
   tc-secrets auth
   ```
2. Then create a new secret on AWS Secrets Manager
3. Use the `sync` command to download the secret to a local file:
   ```
   tc-secrets sync -f .env
   ```
   You will be prompted to select the secret location
4. Make changes to the local `.env` file
5. Update the local secret file version:
   ```
   tc-secrets update -f .env
   ```
6. Finally, synchronize the local changes with AWS Secrets Manager:
   ```
    tc-secrets sync -f .env
    ```
