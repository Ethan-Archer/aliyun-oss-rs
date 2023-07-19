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

- 基础操作

  - [x] 列举存储空间列表 (ListBuckets)
  - [x] 列举 OSS 开服地域信息 (DescribeRegions)

- 存储空间管理

  - [x] 新建存储空间 (PutBucket)
  - [x] 删除存储空间 (DeleteBucket)
  - [x] 列举存储空间内文件列表 (ListObjectsV2)
  - [x] 获取存储空间基本信息 (GetBucketInfo)
  - [x] 获取存储空间统计信息 (GetBucketStat)
  - [x] 批量删除文件 (DeleteMultipleObjects)
  - [x] 列举未完成的分片上传事件 (ListMultipartUploads)

- 文件管理

  - [x] 上传文件 (PutObject)
  - [x] 下载文件 (GetObject)
  - [x] 复制文件 (CopyObject)
  - [x] 追加文件 (AppendObject)
  - [x] 删除文件 (DeleteObject)
  - [x] 解冻文件 (RestoreObject)
  - [x] 获取文件元信息 (HeadObject)
  - [x] 获取文件元信息 (GetObjectMeta)
  - [x] 获取文件访问地址 (GetObjectUrl)
  - 文件分片上传 (MultipartUpload)
    - [x] 初始化分片上传事件 (InitiateMultipartUpload)
    - [x] 上传分片 (UploadPart)
    - [x] 复制文件内容到分片 (UploadPartCopy)
    - [x] 完成分片上传 (CompleteMultipartUpload)
    - [x] 取消分片上传事件 (AbortMultipartUpload)
    - [x] 列举已上传的分片 (ListParts)
  - 文件权限 (ACL)
    - [x] 获取文件权限 (GetObjectACL)
    - [x] 设置文件权限 (PutObjectACL)
  - 文件标签 (Tagging)
    - [x] 获取文件标签 (GetObjectTagging)
    - [x] 设置文件标签 (PutObjectTagging)
    - [x] 清空文件标签 (DeleteObjectTagging)
  - 软链接 (Symlink)
    - [x] 新增软链接 (PutSymlink)
    - [x] 获取软链接 (GetSymlink)
