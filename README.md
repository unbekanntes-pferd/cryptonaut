# Cryptonaut

## What is this?

This is a small tool to automate file key distribution (currently) using the system rescue key.

## How to install?

Download a pre-compiled binary from the releases page in Github or compile from source:

```
# git clone the repo
git clone https://github.com/unbekanntes-pferd/cryptonaut
cargo build --release 
```

Just add the compiled binary to one of your OS paths for executables such as `/usr/local/bin`.

## How to configure?

This tool is intended to be used in a secure location (e.g. some secured virtual machine).

You need to do the following: 

- Create an OAuth application in your DRACOON instance and set it up such that refresh tokens are very long lived
- Create a long lived refresh token with this OAuth application (e.g. via script)
- Activate client-side encryption if not already active and set up system rescue key

To configure the tool, use the [config.yaml.example] example:
- Provide client credentials (id and secret)
- Provide refresh token
- Provide rescue key (system)

## How to use?

```bash
cryptonaut your.dracoon.domain/optional/path

```
This will check if there are any missing file keys for the system rescue key and distribute them.
If you provide a specific path, keys for only this path will be distributed.

### Optional parameters

You can pass the `--debug` flag to activate debug logging.
You can configure the desired path (including file name!) for the log file using `--log-file-path`.
For help with commands, use the `--help` flag.



