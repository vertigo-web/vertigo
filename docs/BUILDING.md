# Troubleshooting installing vertigo-cli

## Performance

Regular way of installing is:

```sh
cargo install --locked --force vertigo-cli
```

### Best performance

```sh
CARGO_PROFILE_RELEASE_LTO=fat cargo install --locked --force vertigo-cli
```

### Low memory systems

```sh
CARGO_PROFILE_RELEASE_LTO=false cargo install --locked --force vertigo-cli
```

### Debugging crashes (of vertigo-cli, not wasm)

```sh
CARGO_PROFILE_RELEASE_DEBUG=true CARGO_PROFILE_RELEASE_STRIP=false cargo install --locked --force vertigo-cli
```

Then:

```sh
RUST_BACKTRACE=1 vertigo watch/serve
```

## Installing on Windows

Usually you need to install OpenSSL first. Use the most up to date **full** version from [https://slproweb.com/products/Win32OpenSSL.html](https://slproweb.com/products/Win32OpenSSL.html).

> [!WARNING]
> Do not install the "light" version.

If for some reason you get problems with compiling SSL, try to pass correct directories:

- PowerShell:

    ```powershell
    $env:OPENSSL_DIR = "C:\Program Files\OpenSSL-Win64"
    $env:OPENSSL_INCLUDE_DIR = "C:\Program Files\OpenSSL-Win64\include"
    $env:OPENSSL_LIB_DIR = "C:\Program Files\OpenSSL-Win64\lib\VC\x64\MT"
    cargo install --locked --force vertigo-cli
    ```

- CMD:

    ```cmd
    set OPENSSL_DIR=C:\Program Files\OpenSSL-Win64
    set OPENSSL_INCLUDE_DIR=C:\Program Files\OpenSSL-Win64\include
    set OPENSSL_LIB_DIR=C:\Program Files\OpenSSL-Win64\lib\VC\x64\MT
    cargo install --locked --force vertigo-cli
    ```
