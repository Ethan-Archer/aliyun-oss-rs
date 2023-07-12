use crate::{common::OssInners, error::normal_error, send::send_to_oss, Error, OssObject};
use bytes::Bytes;
use chrono::NaiveDateTime;
use futures_util::{Stream, StreamExt};
use hyper::{body::to_bytes, Body, Method};
use std::pin::Pin;
use tokio::{
    fs::{create_dir_all, OpenOptions},
    io::{AsyncWriteExt, BufWriter},
};

/// 获取文件内容
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/31980.html)
pub struct GetObject {
    object: OssObject,
    headers: OssInners,
    querys: OssInners,
}
impl GetObject {
    pub fn new(object: OssObject) -> Self {
        GetObject {
            object,
            headers: OssInners::new(),
            querys: OssInners::new(),
        }
    }
    /// 设置版本id
    ///
    /// 只有开启了版本控制时才需要设置
    ///
    pub fn set_version_id(mut self, version_id: impl ToString) -> Self {
        self.querys.insert("versionId", version_id);
        self
    }
    /// 设置响应时的range
    ///
    /// end应该大于等于start，并且两者都在合法索引范围内，如果设置的值不合法，则将下载文件的所有内容
    ///
    /// 文件字节索引是从0开始，例如文件大小是500字节，则索引范围为 0 - 499
    pub fn set_range(mut self, start: usize, end: usize) -> Self {
        self.headers
            .insert("Range", format!("bytes={}-{}", start, end));
        self
    }
    /// 如果指定的时间早于实际修改时间或指定的时间不符合规范，则直接返回Object，并返回200 OK；如果指定的时间等于或者晚于实际修改时间，则返回304 Not Modified。
    ///
    pub fn set_if_modified_since(mut self, if_modified_since: NaiveDateTime) -> Self {
        self.querys.insert(
            "If-Modified-Since",
            if_modified_since.format("%a, %e %b %Y %H:%M:%S GMT"),
        );
        self
    }
    /// 如果指定的时间等于或者晚于Object实际修改时间，则正常传输Object，并返回200 OK；如果指定的时间早于实际修改时间，则返回412 Precondition Failed。
    ///
    pub fn set_if_unmodified_since(mut self, if_unmodified_since: NaiveDateTime) -> Self {
        self.querys.insert(
            "If-Unmodified-Since",
            if_unmodified_since.format("%a, %e %b %Y %H:%M:%S GMT"),
        );
        self
    }
    /// 如果传入的ETag和Object的ETag匹配，则正常传输Object，并返回200 OK；如果传入的ETag和Object的ETag不匹配，则返回412 Precondition Failed。
    ///
    /// Object的ETag值用于验证数据是否发生了更改，您可以基于ETag值验证数据完整性。
    pub fn set_if_match(mut self, if_match: impl ToString) -> Self {
        self.querys.insert("If-Match", if_match);
        self
    }
    /// 如果传入的ETag值和Object的ETag不匹配，则正常传输Object，并返回200 OK；如果传入的ETag和Object的ETag匹配，则返回304 Not Modified。
    ///
    /// Object的ETag值用于验证数据是否发生了更改，您可以基于ETag值验证数据完整性。
    pub fn set_if_none_match(mut self, if_none_match: impl ToString) -> Self {
        self.querys.insert("If-None-Match", if_none_match);
        self
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
        let response = send_to_oss(
            &self.object.client,
            Some(&self.object.bucket),
            Some(&self.object.object),
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
                //读取字节流
                let mut response_bytes = response.into_body();
                while let Some(chunk) = response_bytes.next().await {
                    match chunk {
                        Ok(data) => writer.write_all(&data).await?,
                        Err(e) => return Err(Error::HyperError(e)),
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
        let response = send_to_oss(
            &self.object.client,
            Some(&self.object.bucket),
            Some(&self.object.object),
            Method::GET,
            Some(&self.querys),
            None,
            Body::empty(),
        )?
        .await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => Ok(to_bytes(response.into_body()).await?),
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
        let response = send_to_oss(
            &self.object.client,
            Some(&self.object.bucket),
            Some(&self.object.object),
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
                let stream = response.into_body().map(|item| match item {
                    Ok(bytes) => Ok(bytes),
                    Err(e) => Err(e.into()),
                });
                Ok(Box::pin(stream))
            }
            _ => Err(normal_error(response).await),
        }
    }
}
