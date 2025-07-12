# secrets-cli
A CLI tool for synchronizing .env files with AWS Secrets Manager

### Prerequisites

- AWS CLI installed and configured
- Created an IAM user with permissions to access AWS Secrets Manager
- Created a secret in AWS Secrets Manager, which will be used to store the `.env` file

### Installation

```
cargo install tc-secrets
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

### Remote Secret Structure

The remote secret is defined by a `secret_id` and a `field_id`. The `secret_id` is the identifier for the secret in AWS Secrets Manager, and the `field_id` is the identifier for the specific field within that secret.

With this structure, you can save multiple secret files in a single secret in AWS Secrets Manager, allowing for better organization and management of your secrets.

### Commands

```
>>> tc-secrets -h
A CLI tool for synchronizing .env secret files with AWS Secrets Manager

Usage: tc-secrets <COMMAND>

Commands:
  auth   Authenticate with AWS Secrets Manager
  diff   Display differences between local and remote secret files
  bump   Increment the version of the local secret file
  reset  Reset the local secret file by the remote secret
  sync   Synchronize local secret file with the remote secret
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

```
>>> tc-secrets auth -h
Authenticate with AWS Secrets Manager

Usage: tc-secrets auth

Options:
  -h, --help  Print help
```

```
>>> tc-secrets diff -h
Display differences between local and remote secret files

Usage: tc-secrets diff [OPTIONS] --filepath <FILEPATH>

Options:
  -f, --filepath <FILEPATH>  Path to the local secret file
  -p, --password <PASSWORD>  Optional password for decrypting the secret file [default: secret]
  -h, --help                 Print help
```

```
>>> tc-secrets bump -h
Increment the version of the local secret file

Usage: tc-secrets bump --filepath <FILEPATH>

Options:
  -f, --filepath <FILEPATH>  Path to the local secret file
  -h, --help                 Print help
```

```
>>> tc-secrets reset -h
Reset the local secret file by the remote secret

Usage: tc-secrets reset [OPTIONS] --filepath <FILEPATH>

Options:
  -f, --filepath <FILEPATH>  Path to the local secret file
  -p, --password <PASSWORD>  Optional password for decrypting the secret file [default: secret]
  -h, --help                 Print help
```

```
>>> tc-secrets sync -h
Synchronize local secret file with the remote secret

Usage: tc-secrets sync [OPTIONS] --filepath <FILEPATH>

Options:
  -f, --filepath <FILEPATH>  Path to the local secret file
  -p, --password <PASSWORD>  Optional password for decrypting the secret file [default: secret]
  -h, --help                 Print help
```

