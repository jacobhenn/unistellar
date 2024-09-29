# UniStellar

## API

The entire API is defined and documented in [the wiki](https://github.com/jacobhenn/unistellar/wiki/Internal-API).

## Development

### Configuring the helper

Add a file `unistellar-helper.toml` in `server` and put the following in it:
```toml
# the socket address that the surreal database should bind to
db_addr = "127.0.0.1:8000"

# OPTIONAL: the path to persist the database to.
# i would recommend putting this as a genuine data directory on your machine, e.g.
#    "~/.local/share/unistellar/db" on Linux, or
#    "C:\Users\Name\AppData\Roaming\unistellar\db" on Windows.
# if you want to use a temporary in-memory database, simply comment out this line
db_store_path = "[see comment]"
```

### Using the helper

Run `cargo run --bin helper -- help` *(incl. the space between `--` and `help`)* for a full list of things that the helper can do.

Typically to set up development you will want to use `cargo run --bin helper -- run-db` first to set up the database, then `[..] helper -- run-server` to start the server.

Use `[..] helper -- reset-data` to reset the database with test data.
