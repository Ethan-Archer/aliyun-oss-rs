[![Crates.io](https://img.shields.io/crates/v/aliyun-oss-rs)](https://crates.io/crates/aliyun-oss-rs)
[![Documentation](https://img.shields.io/badge/docs-latest-blue.svg)](https://docs.rs/aliyun-oss-rs)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/EthanWinton/aliyun-oss-rs/blob/main/LICENSE-MIT)

阿里云对象存储服务（Object Storage Service，简称 OSS）的非官方 SDK 实现，无复杂结构设计，链式风格

##### 初始化

```
let client = OssClient::new("Your AccessKey ID","Your AccessKey Secret");
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
let url = object.get_url().url(date);

```

### 已实现接口

<details open>
<summary>基础操作</summary>

- <input type="checkbox" checked disabled> 列举存储空间列表 (ListBuckets)
- <input type="checkbox" checked disabled> 列举 OSS 开服地域信息 (DescribeRegions)

<summary>存储空间管理</summary>

- <input type="checkbox" checked disabled> 新建存储空间 (PutBucket)
- <input type="checkbox" checked disabled> 删除存储空间 (DeleteBucket)
- <input type="checkbox" checked disabled> 列举存储空间内文件列表 (ListObjectsV2)
- <input type="checkbox" checked disabled> 获取存储空间基本信息 (GetBucketInfo)
- <input type="checkbox" checked disabled> 获取存储空间统计信息 (GetBucketStat)
- <input type="checkbox" checked disabled> 批量删除文件 (DeleteMultipleObjects)

<summary>文件管理</summary>

- <input type="checkbox" checked disabled> 上传文件 (PutObject)
- <input type="checkbox" checked disabled> 下载文件 (GetObject)
- <input type="checkbox" checked disabled> 复制文件 (CopyObject)
- <input type="checkbox" checked disabled> 追加文件 (AppendObject)
- <input type="checkbox" checked disabled> 删除文件 (DeleteObject)
- <input type="checkbox" checked disabled> 解冻文件 (RestoreObject)
- <input type="checkbox" checked disabled> 获取文件元信息 (HeadObject)
- <input type="checkbox" checked disabled> 获取文件元信息 (GetObjectMeta)
- <input type="checkbox" checked disabled> 获取文件访问地址 (GetObjectUrl)
- <input type="checkbox" checked disabled> 获取文件标签信息 (GetObjectTagging)

</details>
