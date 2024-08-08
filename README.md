# ch2rs

An auxiliary utility for generating Rust structures from ClickHouse DB schemas for the official [clickhouse-rs](https://github.com/ClickHouse/clickhouse-rs) crate.

## Usage

```sh
cargo install ch2rs
```

### Help

```sh
$ ch2rs --help
```

```
ch2rs 0.1.6

USAGE:
    ch2rs [FLAGS] [OPTIONS] <table>

FLAGS:
    -D               Generate `Deserialize` instances
    -h, --help       Prints help information
        --owned      Generate only owned types
    -S               Generate `Serialize` instances
    -V, --version    Prints version information

OPTIONS:
    -B <bytes>...              Add `#[serde(with = "serde_bytes")]` to the provided column
    -d <database>              A database where the table is placed in [default: default]
    -I <ignore>...             Ignore a specified column
    -O <overrides>...          Override the type of the provided column
    -p <password>
        --derive <trait>...    Add `#[derive(<trait>)]` to the generated types
    -T <types>...              Override the type, e.g. 'Decimal(18, 9)=fixnum::FixedPoint<i64, typenum::U9>'
    -U <url>                   ClickHouse server's URL [default: localhost:8123]
    -u <user>

ARGS:
    <table>    The table's name
```

## Examples

See [snapshots](tests/snapshots).
