use crate::{
    common::{CommonPrefixes, ListBucketResult, Object, OssErrorResponse},
    sign::SignRequest,
    Error, OssBucket,
};
use reqwest::Client;
use std::cmp;

/// 列举存储空间中所有对象的信息
///
/// 默认获取前1000条对象信息，如果需要更多，请设置max_objects
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/187544.html)
pub struct ListObjects {
    bucket: OssBucket,
    list_type: u8,
    delimiter: Option<String>,
    start_after: Option<String>,
    continuation_token: Option<String>,
    max_objects: usize,
    prefix: Option<String>,
    encoding_type: Option<String>,
    fetch_owner: bool,
}

impl ListObjects {
    pub(super) fn new(bucket: OssBucket) -> Self {
        ListObjects {
            bucket,
            list_type: 2,
            delimiter: None,
            start_after: None,
            continuation_token: None,
            max_objects: 1000,
            prefix: None,
            encoding_type: None,
            fetch_owner: false,
        }
    }
    /// 对Object名字进行分组的字符。所有Object名字包含指定的前缀，第一次出现delimiter字符之间的Object作为一组元素（即CommonPrefixes）
    pub fn set_delimiter(mut self, delimiter: &str) -> Self {
        self.delimiter = Some(delimiter.to_owned());
        self
    }
    /// 设定从start-after之后按字母排序开始返回Object。
    ///
    /// start-after用来实现分页显示效果，参数的长度必须小于1024字节。
    ///
    /// 做条件查询时，即使start-after在列表中不存在，也会从符合start-after字母排序的下一个开始打印。
    pub fn set_start_after(mut self, start_after: &str) -> Self {
        self.start_after = Some(start_after.to_owned());
        self
    }
    /// 指定List操作需要从此token开始。
    ///
    /// 可从ListObjects结果中的NextContinuationToken获取此token。
    fn set_continuation_token(&mut self, continuation_token: Option<String>) {
        self.continuation_token = continuation_token;
    }
    /// 限定返回文件的Key必须以prefix作为前缀。
    pub fn set_prefix(mut self, prefix: &str) -> Self {
        self.prefix = Some(prefix.to_owned());
        self
    }
    /// 对返回的内容进行编码并指定编码的类型。
    pub fn set_encoding_type(mut self, encoding_type: &str) -> Self {
        self.encoding_type = Some(encoding_type.to_owned());
        self
    }
    /// 指定返回对象的最大数量。
    ///
    /// 当设置了delimiter时，此参数指的是对象和分组的总和
    ///
    /// 默认值：100，建议不要设置过大的数值，否则返回时间可能会比较长
    pub fn set_max_objects(mut self, max_objects: usize) -> Self {
        let max_objects = cmp::max(1, max_objects);
        self.max_objects = max_objects;
        self
    }
    /// 指定是否在返回结果中包含owner信息。
    pub fn set_fetch_owner(mut self, fetch_owner: bool) -> Self {
        self.fetch_owner = fetch_owner;
        self
    }
    /// 发送请求
    ///
    /// 默认会取回最多1000条结果，如果设置了max_objects，则会取回不多于max_objects的结果数，返回的结果不一定是完整列表，请关注下面的返回值说明。
    ///
    /// 建议不要一次请求过长的结果列表，否则执行时间可能会较长，内网访问的情况下，取回10000条结果，大约需要2秒左右，如果是外网访问，还需要加上网络延时，耗时会大大增加
    ///
    /// - 返回值 0 - 文件列表
    ///
    /// - 返回值 1 - 分组列表
    ///
    /// - 返回值 2 - 列表继续请求的continuation_token，如果不为None，则说明此次请求的结果被 max_objects 截断了，如果还需要获取余下的列表，可以使用此continuation_token再次请求以便获取剩下的结果
    ///
    pub async fn send(
        mut self,
    ) -> Result<(Vec<Object>, Vec<CommonPrefixes>, Option<String>), Error> {
        //构造URL
        let url = format!(
            "https://{}.{}",
            self.bucket.bucket, self.bucket.client.endpoint
        );
        //初始化返回结果
        let mut contents = Vec::new();
        let mut common_prefixes = Vec::new();
        let mut objects_left = self.max_objects;
        while objects_left > 1 {
            //确定本次请求的object数量
            let object_num = cmp::min(objects_left, 1000);
            //构造请求
            let req = Client::new()
                .get(&url)
                .query(&[("list-type", self.list_type)])
                .query(&[("max-keys", object_num)])
                .query(&[("fetch-owner", self.fetch_owner)])
                .query(&[
                    ("delimiter", &self.delimiter),
                    ("start-after", &self.start_after),
                    ("continuation-token", &self.continuation_token),
                    ("prefix", &self.prefix),
                    ("encoding-type", &self.encoding_type),
                ]);
            //发送请求
            let response = req
                .sign(
                    &self.bucket.client.ak_id,
                    &self.bucket.client.ak_secret,
                    Some(&self.bucket.bucket),
                    None,
                )?
                .send()
                .await?;
            //拆解响应消息
            let status_code = response.status();
            let mut total_objects = 0;
            match status_code {
                code if code.is_success() => {
                    let body = response.bytes().await?;
                    let object_list: ListBucketResult = serde_xml_rs::from_reader(&*body)?;
                    if let Some(content) = object_list.contents {
                        let content_num = content.len();
                        contents.reserve(content_num);
                        contents.extend(content);
                        total_objects += content_num;
                    }
                    if let Some(prefixs) = object_list.common_prefixes {
                        let prefixs_num = prefixs.len();
                        common_prefixes.reserve(prefixs_num);
                        common_prefixes.extend(prefixs);
                        total_objects += prefixs_num;
                    }
                    self.set_continuation_token(object_list.next_continuation_token);
                }
                _ => {
                    let body = response.text().await?;
                    let error_info: OssErrorResponse = serde_xml_rs::from_str(&body)?;
                    return Err(Error::OssError(status_code, error_info));
                }
            }
            objects_left -= object_num;
            if total_objects < 1000 {
                break;
            }
        }
        Ok((contents, common_prefixes, self.continuation_token))
    }
}
