use crate::{
    common::{CacheControl, ContentDisposition},
    request::{Oss, OssRequest},
};
use chrono::NaiveDateTime;
use hyper::Method;
use std::net::IpAddr;

/// 获取文件的url
///
/// 私有文件可以通过此方法获取一个授权url，即可直接下载此文件
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/31952.html)
pub struct GetObjectUrl {
    req: OssRequest,
}
impl GetObjectUrl {
    pub(super) fn new(oss: Oss) -> Self {
        GetObjectUrl {
            req: OssRequest::new(oss, Method::GET),
        }
    }
    /// 设置IP信息
    ///
    /// 如果只允许单IP，将subnet_mask设置为32即可
    ///
    pub fn set_source_ip(mut self, source_ip: IpAddr, subnet_mask: u8) -> Self {
        self.req.insert_query("x-oss-ac-source-ip", source_ip);
        self.req
            .insert_query("x-oss-ac-subnet-mask", subnet_mask.to_string());
        self
    }
    /// 设置vpc信息
    ///
    pub fn set_vpc_id(mut self, vpc_id: impl ToString) -> Self {
        self.req.insert_query("x-oss-ac-vpc-id", vpc_id);
        self
    }
    /// 设置允许转发请求
    ///
    /// 默认为不允许
    ///
    pub fn forward_allow(mut self) -> Self {
        self.req.insert_query("x-oss-ac-forward-allow", "true");
        self
    }
    /// 设置响应时的content-type
    ///
    pub fn set_response_mime(
        mut self,
        mime: impl ToString,
        charset: Option<impl ToString>,
    ) -> Self {
        let mut mime_str = mime.to_string();
        if let Some(charset) = charset {
            mime_str.push_str(";charset=");
            mime_str.push_str(&charset.to_string());
        }
        self.req.insert_query("response-content-type", mime_str);
        self
    }
    /// 设置响应时的cache-control
    ///
    pub fn set_response_cache_control(mut self, cache_control: CacheControl) -> Self {
        self.req
            .insert_query("response-cache-control", cache_control);
        self
    }
    /// 设置响应时的content-disposition
    ///
    pub fn set_response_content_disposition(
        mut self,
        content_disposition: ContentDisposition,
    ) -> Self {
        self.req
            .insert_query("response-content-disposition", content_disposition);
        self
    }
    /// 设置自定义域名
    ///
    pub fn set_custom_domain(mut self, custom_domain: impl ToString, enable_https: bool) -> Self {
        self.req.set_endpoint(custom_domain);
        self.req.set_https(enable_https);
        self
    }
    /// 生成url
    ///
    pub fn url(mut self, expires: NaiveDateTime) -> String {
        self.req.query_sign(expires);
        self.req.uri()
    }
}
