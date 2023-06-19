use std::{collections::HashSet, pin::Pin};

use crate::{
    common::{CacheControl, ContentDisposition},
    error::normal_error,
    sign::SignRequest,
    Error, OssObject,
};
use bytes::Bytes;
use chrono::NaiveDateTime;
use futures_util::{Stream, StreamExt};
use mime_guess::Mime;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest::{Client, Response};
use tokio::{
    fs::{create_dir_all, OpenOptions},
    io::{AsyncWriteExt, BufWriter},
};

/// 获取文件的Meta信息
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/31980.html)
pub struct GetObject {
    object: OssObject,
    version_id: Option<String>,
    response_mime: Option<Mime>,
    response_content_language: Option<String>,
    range_start: Option<usize>,
    range_end: Option<usize>,
    response_cache_control: Option<CacheControl>,
    response_content_disposition: Option<ContentDisposition>,
    if_modified_since: Option<NaiveDateTime>,
    if_unmodified_since: Option<NaiveDateTime>,
    if_match: Option<String>,
    if_none_match: Option<String>,
}
impl GetObject {
    pub fn new(object: OssObject) -> Self {
        GetObject {
            object,
            version_id: None,
            response_mime: None,
            response_content_language: None,
            range_start: None,
            range_end: None,
            response_cache_control: None,
            response_content_disposition: None,
            if_modified_since: None,
            if_unmodified_since: None,
            if_match: None,
            if_none_match: None,
        }
    }
    /// 设置版本id
    ///
    /// 只有开启了版本控制时才需要设置
    ///
    pub fn set_version_id(mut self, version_id: &str) -> Self {
        self.version_id = Some(version_id.to_owned());
        self
    }
    /// 设置响应时的content-type
    ///
    pub fn set_response_mime(mut self, response_mime: Mime) -> Self {
        self.response_mime = Some(response_mime);
        self
    }
    /// 设置响应时的content-language
    ///
    pub fn set_response_content_language(mut self, response_content_language: &str) -> Self {
        self.response_content_language = Some(response_content_language.to_owned());
        self
    }
    /// 设置响应时的range
    ///
    /// end如果小于start，end将被直接弃用，没有设置end的情况下，将从start开始下载余下所有内容
    ///
    /// 文件字节索引是从0开始，例如文件大小是500字节，则索引范围为 0 - 499
    pub fn set_range(mut self, start: usize, end: usize) -> Self {
        if end >= start {
            self.range_start = Some(start);
            self.range_end = Some(end);
            self
        } else {
            self.range_start = Some(start);
            self
        }
    }
    /// 设置响应时的cache-control
    ///
    pub fn set_response_cache_control(mut self, response_cache_control: CacheControl) -> Self {
        self.response_cache_control = Some(response_cache_control);
        self
    }
    /// 设置响应时的content-disposition
    ///
    pub fn set_response_content_disposition(
        mut self,
        response_content_disposition: ContentDisposition,
    ) -> Self {
        self.response_content_disposition = Some(response_content_disposition);
        self
    }
    /// 设置响应时的If-Modified-Since
    ///
    /// 如果指定的时间早于实际修改时间或指定的时间不符合规范，则直接返回Object，并返回200 OK；如果指定的时间等于或者晚于实际修改时间，则返回304 Not Modified。
    ///
    pub fn set_if_modified_since(mut self, if_modified_since: NaiveDateTime) -> Self {
        self.if_modified_since = Some(if_modified_since);
        self
    }
    /// 设置响应时的If-Unmodified-Since
    ///
    /// 如果指定的时间等于或者晚于Object实际修改时间，则正常传输Object，并返回200 OK；如果指定的时间早于实际修改时间，则返回412 Precondition Failed。
    ///
    pub fn set_if_unmodified_since(mut self, if_unmodified_since: NaiveDateTime) -> Self {
        self.if_unmodified_since = Some(if_unmodified_since);
        self
    }
    /// 设置响应时的If-Match
    ///
    /// 如果传入的ETag和Object的ETag匹配，则正常传输Object，并返回200 OK；如果传入的ETag和Object的ETag不匹配，则返回412 Precondition Failed。
    ///
    /// Object的ETag值用于验证数据是否发生了更改，您可以基于ETag值验证数据完整性。
    pub fn set_if_match(mut self, if_match: &str) -> Self {
        self.if_match = Some(if_match.to_owned());
        self
    }
    /// 设置响应时的If-None-Match
    ///
    /// 如果传入的ETag值和Object的ETag不匹配，则正常传输Object，并返回200 OK；如果传入的ETag和Object的ETag匹配，则返回304 Not Modified。
    ///
    /// Object的ETag值用于验证数据是否发生了更改，您可以基于ETag值验证数据完整性。
    pub fn set_if_none_match(mut self, if_none_match: &str) -> Self {
        self.if_none_match = Some(if_none_match.to_owned());
        self
    }
    //准备下载
    /// 下载文件保存到磁盘
    ///
    async fn download_ready(self) -> Result<Response, Error> {
        //初始化query
        let mut query = HashSet::new();
        //插入version_id
        if let Some(version_id) = self.version_id {
            query.insert(format!("versionId={}", version_id));
        }
        //插入If-Modified-Since
        if let Some(if_modified_since) = self.if_modified_since {
            query.insert(format!(
                "If-Modified-Since={}",
                if_modified_since
                    .format("%a, %e %b %Y %H:%M:%S GMT")
                    .to_string()
            ));
        }
        //插入If-Unmodified-Since
        if let Some(if_unmodified_since) = self.if_unmodified_since {
            query.insert(format!(
                "If-Unmodified-Since={}",
                if_unmodified_since
                    .format("%a, %e %b %Y %H:%M:%S GMT")
                    .to_string()
            ));
        }
        //插入If-Match
        if let Some(if_match) = self.if_match {
            query.insert(format!("If-Match={}", if_match));
        }
        //插入If-None-Match
        if let Some(if_none_match) = self.if_none_match {
            query.insert(format!("If-None-Match={}", if_none_match));
        }
        //对文件名进行urlencode
        let filename_str = utf8_percent_encode(&self.object.object, NON_ALPHANUMERIC).to_string();
        //构造URL
        let mut url = format!(
            "https://{}.{}/{}",
            self.object.bucket, self.object.client.endpoint, filename_str
        );
        if !query.is_empty() {
            let query_str = query.into_iter().collect::<Vec<_>>().join("&");
            url.push_str("?");
            url.push_str(&query_str);
        }
        //构造请求
        let mut req = Client::new().get(url);
        //插入Range
        if let Some(start) = self.range_start {
            req = req.header(
                "Range",
                format!(
                    "bytes={}-{}",
                    start,
                    self.range_end
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| String::new())
                ),
            );
        }
        //发送请求
        Ok(req
            .sign(
                &self.object.client.ak_id,
                &self.object.client.ak_secret,
                Some(&self.object.bucket),
                Some(&self.object.object),
            )?
            .send()
            .await?)
    }
    /// 下载文件保存到磁盘
    ///
    /// 不支持网络路径，如果需要保存到smb\nfs等网络存储，请先挂载到本地，再使用本地路径地址
    pub async fn download_to_file(self, save_path: &str) -> Result<(), Error> {
        //判断路径
        if save_path.contains("://") {
            return Err(Error::PathNotSupported);
        }
        //发起请求
        let response = self.download_ready().await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => {
                //创建目录
                let parent_dir = std::path::Path::new(save_path).parent();
                if let Some(dir) = parent_dir {
                    create_dir_all(dir).await?;
                }
                //创建文件
                let file = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(save_path)
                    .await?;
                //创建写入缓冲区
                let mut writer = BufWriter::with_capacity(131072, file);
                //将响应转换为字节流
                let mut response_bytes = response.bytes_stream();
                //读取字节流
                while let Some(chunk) = response_bytes.next().await {
                    match chunk {
                        Ok(data) => writer.write_all(&data).await?,
                        Err(e) => return Err(Error::HttpError(e)),
                    }
                }
                writer.flush().await?;
                writer.shutdown().await?;
                Ok(())
            }
            _ => Err(normal_error(response).await),
        }
    }
    /// 下载文件，直接将内容返回
    ///
    /// 如果文件较大，此方法可能占用过多内存，谨慎使用
    pub async fn download_to_buf(self) -> Result<Bytes, Error> {
        //发起请求
        let response = self.download_ready().await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => Ok(response.bytes().await?),
            _ => Err(normal_error(response).await),
        }
    }
    /// 下载文件，返回一个数据流
    ///
    /// 如果文件较大，又不希望直接保存成文件，可以使用此方法，自行对流进行加工
    ///
    /// ```
    /// use futures_util::StreamExt;
    ///
    /// let mut stream = object.get_object().download_to_stream().await.unwrap();
    /// while let Some(item) = stream.next().await {
    ///     match item {
    ///         Ok(bytes) => {
    ///             // Do something with bytes...
    ///         }
    ///         Err(e) => eprintln!("Error: {}", e),
    ///     }
    /// }
    /// ```
    pub async fn download_to_stream(
        self,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<bytes::Bytes, Error>> + Send>>, Error> {
        //发起请求
        let response = self.download_ready().await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => {
                let stream = response.bytes_stream().map(|item| match item {
                    Ok(bytes) => Ok(bytes),
                    Err(e) => Err(e.into()),
                });
                Ok(Box::pin(stream))
            }
            _ => Err(normal_error(response).await),
        }
    }
}
