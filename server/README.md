# UniStellar

## API

The entire API is defined and documented in [`src/routes.rs`](src/routes.rs).

## Development

### Starting the database

Run the equivalent of the following command to start the SurrealDB database:

```shell
# if `PATH` is given, Surreal will persist the database to the given store directory.
#    i would recommend putting this as a genuine data directory on your machine, e.g.
#    "~/.local/share/unistellar/db" on Linux, or
#    "C:\Users\Name\AppData\Roaming\unistellar\db" on Windows.
# if `PATH` is not given, Surreal will use a temporary in-memory database.
#
# if you are connecting to an on-disk database, only put `--user root --pass root` the first time
#     you connect to the database. initializing the root user is not required on subsequent accesses

surreal start rocksdb://[PATH] -A -b 127.0.0.1:8000 [--user root --pass root]
```

Run the following command to load test data from `server/test_data.surql` into the database (make sure the database is running on the correct port):

```shell
# current directory: "server"

surreal import --conn http://127.0.0.1:8000 --ns unistellar --db main test_data.surql
```

### Launching the server

If you have Rust installed, launch the server with the following:

```shell
# current directory: "server"

cargo run -- --db-addr 127.0.0.1:8000
```

For more information about the command-line arguments taken by the server, do `cargo run -- --help`.
