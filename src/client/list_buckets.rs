use crate::{
    common::{ListAllMyBuckets, ListAllMyBucketsResult, OssInners},
    error::normal_error,
    send::send_to_oss,
    Error, OssClient,
};
use hyper::{body::to_bytes, Body, Method};

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
    querys: OssInners,
    headers: OssInners,
}

impl ListBuckets {
    pub(super) fn new(client: OssClient) -> Self {
        ListBuckets {
            client,
            querys: OssInners::new(),
            headers: OssInners::new(),
        }
    }

    /// 限定返回的Bucket名称必须以prefix作为前缀。如果不设定，则不过滤前缀信息。
    ///
    /// 前缀要求：
    /// - 不能为空，长度不能大于63字节
    /// - 只能含有小写英文字母和数字，以及 - 连字符，且不能以连字符开头
    ///
    pub fn set_prefix(mut self, prefix: impl ToString) -> Self {
        self.querys.insert("prefix", prefix);
        self
    }
    /// 设定结果从marker之后按字母排序的第一个开始返回。如果不设定，则从头开始返回数据。
    pub fn set_marker(mut self, marker: impl ToString) -> Self {
        self.querys.insert("marker", marker);
        self
    }
    /// 限定此次返回Bucket的最大个数。取值范围：1~1000，默认值：100
    pub fn set_max_keys(mut self, max_keys: u32) -> Self {
        self.querys.insert("max-keys", max_keys);
        self
    }
    /// 指定资源组ID
    pub fn set_group_id(mut self, group_id: impl ToString) -> Self {
        self.headers.insert("x-oss-resource-group-id", group_id);
        self
    }
    /// 发送请求
    pub async fn send(&self) -> Result<ListAllMyBuckets, Error> {
        //构建http请求
        let response = send_to_oss(
            &self.client,
            None,
            None,
            Method::GET,
            Some(&self.querys),
            Some(&self.headers),
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
                let result: ListAllMyBucketsResult = serde_xml_rs::from_reader(&*response_bytes)
                    .map_err(|_| Error::OssInvalidResponse(Some(response_bytes)))?;
                Ok(ListAllMyBuckets {
                    next_marker: result.next_marker,
                    buckets: result.buckets.bucket,
                })
            }
            _ => Err(normal_error(response).await),
        }
    }
}
