[![Crates.io](https://img.shields.io/crates/v/aliyun-oss-rs)](https://crates.io/crates/aliyun-oss-rs)
[![Documentation](https://img.shields.io/badge/docs-latest-blue.svg)](https://docs.rs/aliyun-oss-rs)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/EthanWinton/aliyun-oss-rs/blob/main/LICENSE-MIT)

阿里云对象存储服务（Object Storage Service，简称 OSS），是阿里云对外提供的海量、安全、低成本、高可靠的云存储服务。

没有复杂的结构，仅仅为快速调用而实现，设计遵循极简、实用原则，通过 OssClient - OssBucket - OssObject 三层结构，实现了部份常用 API，目前不支持的 API 在后续会逐步增加。

目前仅实现了少量常用 API，后续将逐步增加其他 API 支持。

##### 初始化

```
let client = OssClient::new(
"Your AccessKey ID",
"Your AccessKey Secret",
);
```

##### 查询存储空间列表

```
let buckets = client.list_buckets().set_prefix("rust").send().await;
```

##### 查询存储空间中文件列表

```
let bucket = client.bucket("for-rs-test","oss-cn-zhangjiakou.aliyuncs.com")
             .list_objects()
             .set_max_objects(200)
             .set_prefix("rust")
             .send()
             .await;
```

##### 上传文件

```
let object = client.bucket("for-rs-test").object("rust.png");
let result = object.put_object().send_file("Your File Path").await;
```

##### 获取文件访问地址

```
use chrono::{Duration, Local};

let date = Local::now().naive_local() + Duration::days(3);
let url = object.get_url(date).build().await;

```
