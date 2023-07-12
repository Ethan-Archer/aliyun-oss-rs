use crate::{
    common::{url_encode, CacheControl, ContentDisposition},
    OssObject,
};
use base64::{engine::general_purpose, Engine};
use chrono::NaiveDateTime;
use hyper::Method;
use mime_guess::Mime;
use ring::hmac;
use std::{collections::HashMap, net::IpAddr};

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
    charset: Option<String>,
    response_cache_control: Option<CacheControl>,
    response_content_disposition: Option<ContentDisposition>,
    custom_domain: Option<String>,
    enable_https: bool,
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
            charset: None,
            response_cache_control: None,
            response_content_disposition: None,
            custom_domain: None,
            enable_https: true,
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
    pub fn enable_forward_allow(mut self) -> Self {
        self.forward_allow = true;
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
    pub fn set_response_mime(mut self, response_mime: Mime, charset: Option<&str>) -> Self {
        self.response_mime = Some(response_mime);
        self.charset = charset.map(|v| v.to_owned());
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
    /// 设置自定义域名
    ///
    pub fn set_custom_domain(mut self, custom_domain: &str, enable_https: bool) -> Self {
        self.custom_domain = Some(custom_domain.to_owned());
        self.enable_https = enable_https;
        self
    }
    /// 生成url
    ///
    pub fn build(self) -> String {
        //生成CanonicalizedResource
        let mut sub_resource = HashMap::with_capacity(8);
        //增加ip信息
        if let Some(source_ip) = self.source_ip {
            sub_resource.insert("x-oss-ac-source-ip", source_ip.to_string());
        }
        if let Some(subnet_mask) = self.subnet_mask {
            sub_resource.insert("x-oss-ac-subnet-mask", subnet_mask.to_string());
        }
        //增加是否允许转发
        if self.forward_allow {
            sub_resource.insert("x-oss-ac-forward-allow", "true".to_string());
        }
        //增加vpc信息
        if let Some(vpc_id) = self.vpc_id {
            sub_resource.insert("x-oss-ac-vpc-id", vpc_id);
        }
        //增加response-mime
        if let Some(response_mime) = self.response_mime {
            let mut mime_str = response_mime.to_string();
            if let Some(charset) = self.charset {
                mime_str.push_str(";charset=");
                mime_str.push_str(&charset);
            }
            sub_resource.insert("response-content-type", mime_str);
        }
        //插入version_id
        if let Some(version_id) = self.version_id {
            sub_resource.insert("versionId", version_id);
        }
        //插入cache_control
        if let Some(cache_control) = self.response_cache_control {
            sub_resource.insert("response-cache-control", cache_control.to_string());
        }
        //插入cache_control
        if let Some(content_disposition) = self.response_content_disposition {
            sub_resource.insert(
                "response-content-disposition",
                content_disposition.to_string(),
            );
        }
        //组装CanonicalizedResource
        let mut cr_str = sub_resource
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>();
        cr_str.sort();
        let canonicalized_resource = if cr_str.is_empty() {
            format!("/{}/{}", self.object.bucket, self.object.object)
        } else {
            format!(
                "/{}/{}?{}",
                self.object.bucket,
                self.object.object,
                cr_str.join("&")
            )
        };
        //组装待签名字符串
        let unsign_str = format!(
            "{}\n\n\n{}\n{}",
            Method::GET,
            self.expires.timestamp(),
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
        let protocal = if self.enable_https { "https" } else { "http" };
        let host = if let Some(custom_domain) = &self.custom_domain {
            custom_domain.to_string()
        } else {
            format!("{}.{}", self.object.bucket, self.object.client.endpoint)
        };
        sub_resource.remove("x-oss-ac-source-ip");
        if sub_resource.is_empty() {
            format!(
                "{}://{}/{}?Expires={}&OSSAccessKeyId={}&Signature={}",
                protocal,
                host,
                url_encode(&self.object.object),
                self.expires.timestamp(),
                self.object.client.ak_id,
                url_encode(&sign_str)
            )
        } else {
            let query_str = sub_resource
                .into_iter()
                .map(|(k, v)| format!("{}={}", k, url_encode(&v)))
                .collect::<Vec<_>>()
                .join("&");
            format!(
                "{}://{}/{}?Expires={}&OSSAccessKeyId={}&{}&Signature={}",
                protocal,
                host,
                url_encode(&self.object.object),
                self.expires.timestamp(),
                self.object.client.ak_id,
                query_str,
                url_encode(&sign_str)
            )
        }
    }
}
