use crate::{
    common::{BucketBase, ListAllMyBucketsResult},
    error::normal_error,
    sign::SignRequest,
    Error, OssClient,
};
use reqwest::Client;
use std::{borrow::Cow, cmp};

/// 查询存储空间列表
///
/// 可以通过 set_ 方法设置查询过滤条件，具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/31957.html)
///
/// ```
/// let client = OssClient::new("AccessKey ID","AccessKey Secret","oss-cn-beijing.aliyuncs.com");
/// let buckets = client.list_buckets().set_prefix("rust").send().await;
/// println!("{:#?}", buckets);
/// ```
///
pub struct ListBuckets {
    client: OssClient,
    prefix: Option<Cow<'static, str>>,
    marker: Option<Cow<'static, str>>,
    max_keys: u16,
    group_id: Option<Cow<'static, str>>,
}

impl ListBuckets {
    pub(super) fn new(client: &OssClient) -> Self {
        ListBuckets {
            client: client.clone(),
            prefix: None,
            marker: None,
            max_keys: 100,
            group_id: None,
        }
    }

    /// 限定返回的Bucket名称必须以prefix作为前缀。如果不设定，则不过滤前缀信息。
    pub fn set_prefix(mut self, prefix: &str) -> Self {
        self.prefix = Some(Cow::Owned(prefix.to_owned()));
        self
    }

    /// 设定结果从marker之后按字母排序的第一个开始返回。如果不设定，则从头开始返回数据。
    pub fn set_marker(mut self, marker: &str) -> Self {
        self.marker = Some(Cow::Owned(marker.to_owned()));
        self
    }

    /// 限定此次返回Bucket的最大个数。取值范围：1~1000，默认值：100
    pub fn set_max_keys(mut self, max_keys: u16) -> Self {
        let max_keys = cmp::max(1, cmp::min(max_keys, 100));
        self.max_keys = max_keys;
        self
    }
    /// 指定资源组ID
    pub fn set_group_id(mut self, group_id: &str) -> Self {
        self.group_id = Some(Cow::Owned(group_id.to_owned()));
        self
    }
    /// 发送请求
    pub async fn send(&self) -> Result<Vec<BucketBase>, Error> {
        //构建http请求
        let mut req = Client::new()
            .get(format!("https://{}/", self.client.endpoint))
            .query(&[
                ("prefix", self.prefix.as_deref()),
                ("marker", self.marker.as_deref()),
            ])
            .query(&[("max-keys", self.max_keys)]);
        //附加header
        if let Some(group_id) = &self.group_id {
            req = req.header("x-oss-resource-group-id", group_id.as_ref());
        }
        //发送请求
        let response = req
            .sign(&self.client.ak_id, &self.client.ak_secret, None, None)?
            .send()
            .await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => {
                let response_bytes = response
                    .bytes()
                    .await
                    .map_err(|_| Error::OssInvalidResponse(None))?;
                let buckets: ListAllMyBucketsResult =
                    serde_xml_rs::from_reader(&*response_bytes)
                        .map_err(|_| Error::OssInvalidResponse(Some(response_bytes.into())))?;
                Ok(buckets.buckets.bucket)
            }
            _ => Err(normal_error(response).await),
        }
    }
}
