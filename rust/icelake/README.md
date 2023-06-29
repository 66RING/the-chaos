# icelake

> The Rust implementation of [Iceberg](https://iceberg.apache.org/).

一种数据存储和表示的格式, 如json等。

纯API操作, pass

## Usage

```rust
// open
let table = Table::open(table_uri.as_str()).await?;
// read metadata
println!("{:#?}", table.current_table_metadata());
```

利用文件系统作为磁盘存储

```
. (uri)
├── data
│   ├── 00000-0-b8982382-f016-467a-84e4-5e6bbe0ff19a-00001.parquet
│   ├── 00001-1-b8982382-f016-467a-84e4-5e6bbe0ff19a-00001.parquet
│   └── 00002-2-b8982382-f016-467a-84e4-5e6bbe0ff19a-00001.parquet
└── metadata
    ├── 10d28031-9739-484c-92db-cdf2975cead4-m0.avro
    ├── snap-1646658105718557341-1-10d28031-9739-484c-92db-cdf2975cead4.avro
    ├── v1.metadata.json
    ├── v2.metadata.json
    └── version-hint.text // 版本信息
```

## impl

- open
    * 利用opendal打开uri, 构造opendal Operator
    * 然后使用Operator读取文件系统信息, 如`metadata/version-hint.text`存储了版本信息
- misc: 其他操作也差不多
    * json文件的就用`serde::from_json`解析
    * avro文件的就用`apache_avro` crate





