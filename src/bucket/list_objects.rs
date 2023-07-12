use crate::{
    common::{OssInners, Owner, StorageClass},
    error::normal_error,
    send::send_to_oss,
    Error, OssBucket,
};
use hyper::{body::to_bytes, Body, Method};
use serde_derive::Deserialize;
use std::cmp;

// 返回内容
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ObjectsList {
    // 列表继续请求的token
    pub next_continuation_token: Option<String>,
    // 文件列表
    pub contents: Option<Vec<ObjectInfo>>,
    // 分组列表
    pub common_prefixes: Option<Vec<CommonPrefixes>>,
}

/// Object文件信息
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ObjectInfo {
    /// Object路径
    pub key: String,
    /// Object最后修改时间
    pub last_modified: String,
    /// ETag在每个Object生成时创建，用于标识一个Object的内容，ETag值可以用于检查Object内容是否发生变化，不建议使用ETag值作为Object内容的MD5校验数据完整性的依据。
    pub e_tag: String,
    #[serde(rename = "Type")]
    pub type_field: String,
    /// Object大小，单位为字节
    pub size: u64,
    /// Object的存储类型
    pub storage_class: StorageClass,
    /// Object的解冻状态
    pub restore_info: Option<String>,
    /// Bucket拥有者信息
    pub owner: Option<Owner>,
}

/// 分组列表
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CommonPrefixes {
    /// 前缀
    pub prefix: String,
}
/// 列举存储空间中所有文件的信息
///
/// 默认获取前1000条文件信息
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/187544.html)
pub struct ListObjects {
    bucket: OssBucket,
    querys: OssInners,
}

impl ListObjects {
    pub(super) fn new(bucket: OssBucket) -> Self {
        let mut querys = OssInners::from("list-type", "2");
        querys.insert("max-keys", "1000");
        ListObjects { bucket, querys }
    }
    /// 对Object名字进行分组的字符。所有Object名字包含指定的前缀，第一次出现delimiter字符之间的Object作为一组元素（即CommonPrefixes）
    pub fn set_delimiter(mut self, delimiter: impl ToString) -> Self {
        self.querys.insert("delimiter", delimiter);
        self
    }
    /// 设定从start-after之后按字母排序开始返回Object。
    ///
    /// start-after用来实现分页显示效果，参数的长度必须小于1024字节。
    ///
    /// 做条件查询时，即使start-after在列表中不存在，也会从符合start-after字母排序的下一个开始。
    pub fn set_start_after(mut self, start_after: impl ToString) -> Self {
        self.querys.insert("start-after", start_after);
        self
    }
    /// 指定List操作需要从此token开始。
    ///
    /// 可从ListObjects结果中的NextContinuationToken获取此token。
    pub fn set_continuation_token(mut self, continuation_token: impl ToString) -> Self {
        self.querys.insert("continuation-token", continuation_token);
        self
    }
    /// 限定返回文件的Key必须以prefix作为前缀。
    pub fn set_prefix(mut self, prefix: impl ToString) -> Self {
        self.querys.insert("prefix", prefix.to_string());
        self
    }
    /// 指定返回文件的最大数量。
    ///
    /// 当设置了delimiter时，此参数指的是文件和分组的总和
    ///
    /// 默认值：1000，取值范围：1 - 1000，设置的值如不在这个范围，则会使用默认值
    pub fn set_max_keys(mut self, max_keys: u32) -> Self {
        let max_keys = cmp::min(1000, cmp::max(1, max_keys));
        self.querys.insert("max-keys", max_keys);
        self
    }
    /// 指定是否在返回结果中包含owner信息。
    pub fn fetch_owner(mut self) -> Self {
        self.querys.insert("fetch-owner", "true");
        self
    }
    /// 发送请求
    ///
    pub async fn send(self) -> Result<ObjectsList, Error> {
        //构建http请求
        let response = send_to_oss(
            &self.bucket.client,
            Some(&self.bucket.bucket),
            None,
            Method::GET,
            Some(&self.querys),
            None,
            Body::empty(),
        )?
        .await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => {
                let response_bytes = to_bytes(response.into_body())
                    .await
                    .map_err(|_| Error::OssInvalidResponse(None))?;
                let object_list: ObjectsList = serde_xml_rs::from_reader(&*response_bytes)
                    .map_err(|_| Error::OssInvalidResponse(Some(response_bytes)))?;
                Ok(object_list)
            }
            _ => return Err(normal_error(response).await),
        }
    }
}
