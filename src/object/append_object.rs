use crate::{
    common::{Acl, CacheControl, ContentDisposition, OssErrorResponse, StorageClass},
    error::Error,
    sign::SignRequest,
    OssObject,
};
use futures::stream::StreamExt;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest::{header, Body, Client};
use std::collections::{HashMap, HashSet};
use tokio::{fs::File, io::BufReader};
use tokio_util::io::ReaderStream;

/// 追加文件
///
/// 只允许对类型为Appendable的文件进行追加，通过put方法上传的文件不允许追加
///
/// 追加文件时，文件的最终大小不允许超过 5GB
///
/// 追加文件的逻辑和限制较为复杂，建议仔细阅读 [阿里云官方文档](https://help.aliyun.com/document_detail/31978.html)
pub struct AppendObject {
    object: OssObject,
    position: u32,
    mime: Option<String>,
    acl: Option<Acl>,
    storage_class: Option<StorageClass>,
    cache_control: Option<CacheControl>,
    content_disposition: Option<ContentDisposition>,
    forbid_overwrite: bool,
    x_oss_meta: HashMap<String, String>,
    x_oss_tagging: HashSet<String>,
    callback: Option<Box<dyn Fn(u64, u64) + Send + Sync + 'static>>,
}

impl AppendObject {
    pub(super) fn new(object: OssObject) -> Self {
        AppendObject {
            object,
            position: 0,
            mime: None,
            acl: None,
            storage_class: None,
            cache_control: None,
            content_disposition: None,
            forbid_overwrite: false,
            x_oss_meta: HashMap::new(),
            x_oss_tagging: HashSet::new(),
            callback: None,
        }
    }
    /// 设置追加内容的起点
    pub fn set_position(mut self, position: u32) -> Self {
        self.position = position;
        self
    }
    /// 设置对象的mime类型
    pub fn set_mime(mut self, mime: &str) -> Self {
        self.mime = Some(mime.to_owned());
        self
    }
    /// 设置对象的访问权限
    pub fn set_acl(mut self, acl: Acl) -> Self {
        self.acl = Some(acl);
        self
    }
    /// 设置对象的存储类型
    pub fn set_storage_class(mut self, storage_class: StorageClass) -> Self {
        self.storage_class = Some(storage_class);
        self
    }
    /// 对象被下载时网页的缓存行为
    pub fn set_cache_control(mut self, cache_control: CacheControl) -> Self {
        self.cache_control = Some(cache_control);
        self
    }
    /// 设置对象的展示形式
    pub fn set_content_disposition(mut self, content_disposition: ContentDisposition) -> Self {
        self.content_disposition = Some(content_disposition);
        self
    }
    /// 设置是否允许覆盖同名对象
    pub fn set_forbid_overwrite(mut self, forbid_overwrite: bool) -> Self {
        self.forbid_overwrite = forbid_overwrite;
        self
    }
    /// 设置需要附加的metadata
    pub fn set_meta(mut self, key: &str, value: &str) -> Self {
        self.x_oss_meta.insert(key.to_owned(), value.to_owned());
        self
    }
    /// 设置tag信息
    pub fn set_tagging(mut self, key: &str, value: Option<&str>) -> Self {
        match value {
            Some(v) => self.x_oss_tagging.insert(format!("{}={}", key, v)),
            None => self.x_oss_tagging.insert(key.to_owned()),
        };
        self
    }
    /// 设置对象上传进度的回调方法，此方法仅对send_file()有效
    /// ```
    /// let callback = Box::new(|uploaded_size: u64, total_size: u64| {
    ///     let percentage = if total_size == 0 {
    ///         100.0
    ///     } else {
    ///         (uploaded_size as f64) / (total_size as f64) * 100.00
    ///     };
    ///     println!("{:.2}%", percentage);
    /// });
    /// ```
    pub fn set_callback(mut self, callback: Box<dyn Fn(u64, u64) + Send + Sync + 'static>) -> Self {
        self.callback = Some(callback);
        self
    }
    /// 将磁盘中的文件上传到OSS
    ///
    /// 如果设置了上传进度的回调方法，调用者将会实时获得最新的上传进度
    ///
    /// - 返回值 0 - OSS返回的position标识，下一次如果要继续追加文件，需要从此position开始
    /// - 返回值 1 - OSS返回的CRC64结果
    /// - 返回值 2 - 开启版本控制时，OSS会返回的文件版本号
    pub async fn send_file(
        self,
        file: &str,
    ) -> Result<(Option<String>, Option<String>, Option<String>), Error> {
        //生成文件类型
        let file_type = match self.mime {
            Some(mime) => mime,
            None => match infer::get_from_path(file)? {
                Some(ext) => ext.mime_type().to_owned(),
                None => mime_guess::from_path(&*self.object.object)
                    .first()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "application/octet-stream".to_owned())
                    .to_string(),
            },
        };
        //打开文件
        let file = File::open(file).await?;
        //读取文件大小
        let file_size = file.metadata().await?.len();
        if file_size >= 5_000_000_000 {
            return Err(Error::FileTooBig);
        }
        //对文件名进行urlencode
        let filename_str = utf8_percent_encode(&self.object.object, NON_ALPHANUMERIC).to_string();
        //构造url
        let url = format!(
            "https://{}.{}/{}?append",
            self.object.bucket, self.object.client.endpoint, filename_str
        );
        //构造http请求
        let mut req = Client::new()
            .post(url)
            //插入追加起点
            .query(&[("position", self.position)])
            //插入content_type
            .header(header::CONTENT_TYPE, file_type)
            //插入content_length
            .header(header::CONTENT_LENGTH, file_size);
        //插入acl
        if let Some(acl) = self.acl {
            req = req.header("x-oss-object-acl", acl.to_string());
        }
        //插入cache-control
        if let Some(cache_control) = self.cache_control {
            req = req.header(header::CACHE_CONTROL, cache_control.to_string());
        }
        //插入content-disposition
        if let Some(content_disposition) = self.content_disposition {
            req = req.header(header::CONTENT_DISPOSITION, content_disposition.to_string());
        }
        //插入存储类型
        if let Some(storage_class) = self.storage_class {
            req = req.header("x-oss-storage-class", storage_class.to_string());
        }
        //插入不允许覆盖标志
        if self.forbid_overwrite {
            req = req.header("x-oss-forbid-overwrite", "true");
        }
        //插入x-oss-meta
        for (key, value) in self.x_oss_meta {
            req = req.header("x-oss-meta-".to_owned() + &key, value)
        }
        //插入x-oss-tagging
        if !self.x_oss_tagging.is_empty() {
            let tagging_str = self
                .x_oss_tagging
                .into_iter()
                .collect::<Vec<String>>()
                .join("&");
            req = req.header("x-oss-tagging", tagging_str);
        }
        //初始化文件内容读取数据流
        let buf = BufReader::with_capacity(131072, file);
        let stream = ReaderStream::with_capacity(buf, 16384);
        //初始化已上传内容大小
        let mut uploaded_size = 0;
        //初始化上传请求
        let req = req.body(Body::wrap_stream(stream.map(move |result| match result {
            Ok(chunk) => {
                if let Some(callback) = &self.callback {
                    let upload_size = chunk.len() as u64;
                    uploaded_size = uploaded_size + upload_size;
                    callback(uploaded_size, file_size);
                }
                Ok(chunk)
            }
            Err(err) => Err(err),
        })));
        //上传文件
        let response = req
            .sign(
                &self.object.client.ak_id,
                &self.object.client.ak_secret,
                Some(&self.object.bucket),
                Some(&self.object.object),
            )?
            .send()
            .await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => {
                let headers = response.headers();
                let crc64ecma = headers
                    .get("x-oss-hash-crc64ecma")
                    .and_then(|header| header.to_str().ok().map(|s| s.to_owned()))
                    .map(|v| v.replace("\"", ""));
                let version_id = headers
                    .get("x-oss-version-id")
                    .and_then(|header| header.to_str().ok().map(|s| s.to_owned()));
                let next_position = headers
                    .get("x-oss-next-append-position")
                    .and_then(|header| header.to_str().ok().map(|s| s.to_owned()));
                Ok((next_position, crc64ecma, version_id))
            }
            _ => {
                let body = response.text().await?;
                let error_info: OssErrorResponse = serde_xml_rs::from_str(&body)?;
                return Err(Error::OssError(status_code, error_info));
            }
        }
    }
    /// 将内存中的数据上传到OSS
    ///
    /// - 返回值 0 - OSS返回的position标识，下一次如果要继续追加文件，需要从此position开始
    /// - 返回值 1 - OSS返回的CRC64结果
    /// - 返回值 2 - 开启版本控制时，OSS会返回的文件版本号
    pub async fn send_content(
        self,
        content: &[u8],
    ) -> Result<(Option<String>, Option<String>, Option<String>), Error> {
        //生成文件类型
        let content_type = self.mime.unwrap_or_else(|| match infer::get(content) {
            Some(ext) => ext.mime_type().to_owned(),
            None => mime_guess::from_path(&*self.object.object)
                .first()
                .map(|v| v.to_string())
                .unwrap_or_else(|| "application/octet-stream".to_owned())
                .to_string(),
        });
        //读取文件大小
        let content_size = content.len() as u64;
        if content_size >= 5_000_000_000 {
            return Err(Error::FileTooBig);
        }
        //对文件名进行urlencode
        let filename_str = utf8_percent_encode(
            &self.object.object.trim().trim_matches('/'),
            NON_ALPHANUMERIC,
        )
        .to_string();
        //构造请求url
        let url = format!(
            "https://{}.{}/{}?append",
            self.object.bucket, self.object.client.endpoint, filename_str
        );
        //构造http请求
        let mut req = Client::new()
            .post(url)
            //插入追加起点
            .query(&[("position", self.position)])
            .header(header::CONTENT_TYPE, content_type)
            .header(header::CONTENT_LENGTH, content_size)
            .body(content.to_owned());
        //插入acl
        if let Some(acl) = self.acl {
            req = req.header("x-oss-object-acl", acl.to_string());
        }
        //插入cache—control
        if let Some(cache_control) = self.cache_control {
            req = req.header(header::CACHE_CONTROL, cache_control.to_string());
        }
        //插入content-disposition
        if let Some(content_disposition) = self.content_disposition {
            req = req.header(header::CONTENT_DISPOSITION, content_disposition.to_string());
        }
        //插入存储类型
        if let Some(storage_class) = self.storage_class {
            req = req.header("x-oss-storage-class", storage_class.to_string());
        }
        //插入不允许覆盖标志
        if self.forbid_overwrite {
            req = req.header("x-oss-forbid-overwrite", "true");
        }
        //插入x-oss-meta
        for (key, value) in self.x_oss_meta {
            req = req.header("x-oss-meta-".to_owned() + &key, value)
        }
        //插入x-oss-tagging
        if !self.x_oss_tagging.is_empty() {
            let tagging_str = self
                .x_oss_tagging
                .into_iter()
                .collect::<Vec<String>>()
                .join("&");
            req = req.header("x-oss-tagging", tagging_str);
        }
        //签名并发送请求
        let response = req
            .sign(
                &self.object.client.ak_id,
                &self.object.client.ak_secret,
                Some(&self.object.bucket),
                Some(&self.object.object),
            )?
            .send()
            .await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => {
                let headers = response.headers();
                let crc64ecma = headers
                    .get("x-oss-hash-crc64ecma")
                    .and_then(|header| header.to_str().ok().map(|s| s.to_owned()))
                    .map(|v| v.replace("\"", ""));
                let version_id = headers
                    .get("x-oss-version-id")
                    .and_then(|header| header.to_str().ok().map(|s| s.to_owned()));
                let next_position = headers
                    .get("x-oss-next-append-position")
                    .and_then(|header| header.to_str().ok().map(|s| s.to_owned()));
                Ok((next_position, crc64ecma, version_id))
            }
            _ => {
                let body = response.text().await?;
                let error_info: OssErrorResponse = serde_xml_rs::from_str(&body)?;
                return Err(Error::OssError(status_code, error_info));
            }
        }
    }
}
