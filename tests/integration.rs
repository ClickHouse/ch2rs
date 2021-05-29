use std::{
    fs::{self, File},
    io::{self, Write},
};

use clickhouse::Client;
use structopt::StructOpt;

const URL: &str = "http://localhost:8123";

// TODO: check `Map`.
const CREATE_TABLE_DDL: &str = "
    CREATE TABLE ch2rs_test (
        u8          UInt8,
        u16         UInt16,
        u32         UInt32,
        u64         UInt64,
        i8          Int8,
        i16         Int16,
        i32         Int32,
        i64         Int64,
        str         String,
        low_str     LowCardinality(String),
        blob        String,
        fs          FixedString(5),
        f32         Float32,
        f64         Float64,
        d           Date,
        dt          DateTime,
        dt64        DateTime64(9),
        ipv4        IPv4,
        ipv6        IPv6,
        uuid        UUID,
        dec64       Decimal64(9),
        enum8       Enum8('' = -128, 'Foo Bar' = 0),
        enum16      Enum16('' = -128, 'fooBar' = 1024),
        array       Array(LowCardinality(String)),
        tuple       Tuple(String, LowCardinality(String)),
        opt_str     Nullable(String),

        default     DEFAULT u16,
        material    MATERIALIZED u16,
        alias       ALIAS u16
    )
        ENGINE = MergeTree
        ORDER BY u8
";

async fn recreate_table() {
    let client = Client::default().with_url(URL);

    client
        .query("DROP TABLE IF EXISTS ch2rs_test")
        .execute()
        .await
        .expect("failed to drop an old table");

    client
        .query(CREATE_TABLE_DDL)
        .execute()
        .await
        .expect("failed to create a table");
}

async fn run_one(args: Vec<&str>) {
    let options = ch2rs::Options::from_iter(args);
    let code = ch2rs::generate(options)
        .await
        .expect("failed to generate a struct");
    insta::assert_snapshot!(code);
}

async fn generate_all() {
    for t1 in &["-S", "-D", "-SD"] {
        for t2 in &["--owned", ""] {
            let args = vec![
                "ch2rs",
                "ch2rs_test",
                "-U",
                URL,
                t1,
                t2,
                "-T",
                "FixedString(5)=[u8; 5]",
                "-T",
                "Date=u16",
                "-T",
                "DateTime=u32",
                "-T",
                "DateTime64(9)=u64",
                "-T",
                "IPv4=u32",
                "-T",
                "IPv6=[u8; 16]",
                "-T",
                "UUID=[u8; 16]",
                "-T",
                "Decimal(18, 9)=u64",
                "-O",
                "bytes=Vec\\<u8\\>",
            ];
            let args = args.into_iter().filter(|s| !s.is_empty()).collect();
            let tmp = format!("{}_{}", t1, t2);
            let suffix = tmp.trim_end_matches('_');

            let mut settings = insta::Settings::clone_current();
            settings.set_snapshot_suffix(suffix);
            settings.set_prepend_module_to_snapshot(false);
            settings.bind_async(run_one(args)).await;
        }
    }
}

fn extract_code_from_snapshots() {
    match fs::remove_dir_all("target/snapshots") {
        Ok(_) => {}
        Err(err) if err.kind() == io::ErrorKind::NotFound => {}
        Err(err) => panic!("failed to remove target/snapshots: {}", err),
    }

    fs::create_dir_all("target/snapshots").expect("failed to create tests/snapshots");

    for entry in fs::read_dir("tests/snapshots").expect("failed to read tests/snapshots") {
        let entry = entry.expect("invalid entry");
        let content = fs::read_to_string(entry.path()).expect("failed to read the snapshot file");
        let code = content.rsplit("---").next().expect("invalid snapshot");
        let mut file = File::create(format!(
            "target/snapshots/{}.rs",
            entry.file_name().to_string_lossy()
        ))
        .expect("failed to create the source file");

        // TODO: connect to CH in tests.
        file.write_all(format!("{}\nfn main() {{}}", code).as_bytes())
            .expect("failed to write to the source file");
    }
}

fn compile_snapshots() {
    let t = trybuild::TestCases::new();
    t.pass("target/snapshots/*.rs");
}

#[tokio::test]
async fn all() {
    recreate_table().await;
    generate_all().await;
    extract_code_from_snapshots();
    compile_snapshots();
}
