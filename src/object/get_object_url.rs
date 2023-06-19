use crate::{
    common::{CacheControl, ContentDisposition},
    OssObject,
};
use base64::{engine::general_purpose, Engine};
use chrono::NaiveDateTime;
use mime_guess::Mime;
use reqwest::{Method, Url};
use ring::hmac;
use std::{collections::BTreeSet, net::IpAddr};

/// 获取文件的url
///
/// 私有文件可以通过此方法获取一个授权url，即可直接下载此文件
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/31952.html)
pub struct GetObjectUrl {
    object: OssObject,
    expires: NaiveDateTime,
    source_ip: Option<IpAddr>,
    subnet_mask: Option<u8>,
    vpc_id: Option<String>,
    forward_allow: bool,
    version_id: Option<String>,
    response_mime: Option<Mime>,
    response_content_language: Option<String>,
    response_cache_control: Option<CacheControl>,
    response_content_disposition: Option<ContentDisposition>,
    if_modified_since: Option<NaiveDateTime>,
    if_unmodified_since: Option<NaiveDateTime>,
    if_match: Option<String>,
    if_none_match: Option<String>,
}
impl GetObjectUrl {
    pub(super) fn new(object: OssObject, expires: NaiveDateTime) -> Self {
        GetObjectUrl {
            object,
            expires,
            source_ip: None,
            subnet_mask: None,
            vpc_id: None,
            forward_allow: false,
            version_id: None,
            response_mime: None,
            response_content_language: None,
            response_cache_control: None,
            response_content_disposition: None,
            if_modified_since: None,
            if_unmodified_since: None,
            if_match: None,
            if_none_match: None,
        }
    }
    /// 设置IP信息
    ///
    /// 如果只允许单IP，将subnet_mask设置为32即可
    ///
    pub fn set_ip(mut self, source_ip: IpAddr, subnet_mask: u8) -> Self {
        self.source_ip = Some(source_ip);
        self.subnet_mask = Some(subnet_mask);
        self
    }
    /// 设置vpc信息
    ///
    pub fn set_vpc_id(mut self, vpc_id: &str) -> Self {
        self.vpc_id = Some(vpc_id.to_owned());
        self
    }
    /// 设置是否允许转发请求
    ///
    /// 默认为不允许
    ///
    pub fn set_forward_allow(mut self, forward_allow: bool) -> Self {
        self.forward_allow = forward_allow;
        self
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
    /// 生成url
    ///
    pub async fn build(self) -> Option<String> {
        //初始化CanonicalizedOSSHeaders
        let canonicalized_ossheaders = String::new();
        //生成CanonicalizedResource
        let mut canonicalized_resource = format!("/{}/{}", self.object.bucket, self.object.object);
        let mut sub_resource: BTreeSet<String> = BTreeSet::new();
        //初始化URL
        let mut query = format!(
            "Expires={}&OSSAccessKeyId={}",
            self.expires.timestamp(),
            self.object.client.ak_id
        );
        //增加ip信息
        if let Some(subnet_mask) = self.subnet_mask {
            sub_resource.insert(format!("x-oss-ac-subnet-mask={}", subnet_mask));
        }
        //增加vpc信息
        if let Some(vpc_id) = self.vpc_id {
            sub_resource.insert(format!("x-oss-ac-vpc-id={}", vpc_id));
        }
        //增加是否允许转发
        if self.forward_allow {
            sub_resource.insert("x-oss-ac-forward-allow=true".to_owned());
        }
        //增加response-mime
        if let Some(response_mime) = self.response_mime {
            sub_resource.insert(format!(
                "response-content-type={}",
                response_mime.to_string()
            ));
        }
        //组装URL和CanonicalizedResource
        if !sub_resource.is_empty() {
            let push_str = sub_resource.into_iter().collect::<Vec<_>>().join("&");
            query.push_str(&format!("&{}", push_str));
            canonicalized_resource.push_str(&format!("?{}", push_str));
            //CanonicalizedResource增加source_ip
            if let Some(source_ip) = self.source_ip {
                canonicalized_resource.push_str(&format!("&x-oss-ac-source-ip={}", source_ip));
            }
        };
        //组装待签名字符串
        let unsign_str = format!(
            "{}\n\n\n{}\n{}{}",
            Method::GET,
            self.expires.timestamp(),
            canonicalized_ossheaders,
            canonicalized_resource
        );
        //生成签名
        let sign_result = hmac::Key::new(
            hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY,
            self.object.client.ak_secret.as_bytes(),
        );
        let sign_str =
            general_purpose::STANDARD.encode(hmac::sign(&sign_result, unsign_str.as_bytes()));
        //组装url
        query.push_str("&Signature=");
        query.push_str(&sign_str);
        let url = format!(
            "https://{}.{}/{}?{}",
            self.object.bucket, self.object.client.endpoint, self.object.object, query
        );
        Url::parse(&url).ok().map(|v| v.to_string())
    }
}
