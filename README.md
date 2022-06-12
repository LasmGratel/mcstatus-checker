# mcstatus-checker
Check your Minecraft server status, built with rocket.

## Endpoint

`/<server>(:<port>)` will respond a plain text status of `Online` or `Offline`.

`/<server>(:<port>)/json` will respond a detailed JSON status.

## Configuration

[Rocket.rs reference](https://rocket.rs/v0.5-rc/guide/configuration/)

Create a Rocket.toml in your working directory with content:
```toml
[default]
address = "0.0.0.0"
port = 8000
```

This will setup a server listening on `0.0.0.0:8000`
