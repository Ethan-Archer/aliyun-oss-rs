use crate::{
    common::{Acl, DataRedundancyType, OssInners, StorageClass},
    error::normal_error,
    send::send_to_oss,
    Error, OssBucket,
};
use hyper::Method;
use serde_derive::Serialize;
use serde_xml_rs::to_string;

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct CreateBucketConfiguration {
    storage_class: Option<StorageClass>,
    data_redundancy_type: Option<DataRedundancyType>,
}

/// 调用PutBucket接口创建存储空间
///
/// 同一阿里云账号在同一地域（Region）内最多支持创建100个存储空间
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/31959.html)
pub struct PutBucket {
    bucket: OssBucket,
    headers: OssInners,
    storage_class: Option<StorageClass>,
    data_redundancy_type: Option<DataRedundancyType>,
}
impl PutBucket {
    pub(super) fn new(bucket: OssBucket) -> Self {
        PutBucket {
            bucket,
            headers: OssInners::new(),
            storage_class: None,
            data_redundancy_type: None,
        }
    }
    /// 设置存储空间的访问权限
    pub fn set_acl(mut self, acl: Acl) -> Self {
        self.headers.insert("x-oss-acl", acl);
        self
    }
    /// 指定资源组ID
    ///
    /// 如果在请求中携带该请求头并指定资源组ID，则创建的存储空间属于该资源组。当指定的资源组ID为rg-default-id时，创建的存储空间属于默认资源组。
    ///
    /// 如果在请求中未携带该请求头，则创建的存储空间属于默认资源组。
    pub fn set_group_id(mut self, group_id: impl ToString) -> Self {
        self.headers.insert("x-oss-resource-group-id", group_id);
        self
    }
    /// 设置存储空间的存储类型
    pub fn set_storage_class(mut self, storage_class: StorageClass) -> Self {
        self.storage_class = Some(storage_class);
        self
    }
    /// 设置存储空间的数据容灾类型
    pub fn set_redundancy_type(mut self, redundancy_type: DataRedundancyType) -> Self {
        self.data_redundancy_type = Some(redundancy_type);
        self
    }
    /// 发送请求
    pub async fn send(self) -> Result<(), Error> {
        let mut body = String::new();
        if self.data_redundancy_type.is_some() || self.storage_class.is_some() {
            let bucket_config = CreateBucketConfiguration {
                storage_class: self.storage_class,
                data_redundancy_type: self.data_redundancy_type,
            };
            if let Ok(body_str) = to_string(&bucket_config) {
                body.push_str(&body_str)
            };
        }
        //构建http请求
        let response = send_to_oss(
            &self.bucket.client,
            Some(&self.bucket.bucket),
            None,
            Method::PUT,
            None,
            Some(&self.headers),
            body.into(),
        )?
        .await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => Ok(()),
            _ => Err(normal_error(response).await),
        }
    }
}
