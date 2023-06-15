use crate::OssObject;
use base64::{engine::general_purpose, Engine};
use chrono::NaiveDateTime;
use reqwest::{Method, Url};
use ring::hmac;
use std::{collections::BTreeSet, net::IpAddr};

/// 获取文件的url
///
/// 私有文件可以通过此方法获取一个授权url，即可直接下载此文件
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/31952.html)
pub struct GetUrl {
    object: OssObject,
    expires: NaiveDateTime,
    source_ip: Option<IpAddr>,
    subnet_mask: Option<u8>,
    vpc_id: Option<String>,
    forward_allow: bool,
}
impl GetUrl {
    pub(super) fn new(object: OssObject, expires: NaiveDateTime) -> Self {
        GetUrl {
            object,
            expires,
            source_ip: None,
            subnet_mask: None,
            vpc_id: None,
            forward_allow: false,
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
    /// 生成url
    ///
    pub async fn build(self) -> Option<String> {
        //初始化CanonicalizedOSSHeaders
        let canonicalized_ossheaders = String::new();
        //生成CanonicalizedResource
        let mut canonicalized_resource = format!("/{}/{}", self.object.bucket, self.object.object);
        let mut sub_resource: BTreeSet<String> = BTreeSet::new();
        //初始化URL
        let mut param = format!(
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
        //组装URL和CanonicalizedResource
        if !sub_resource.is_empty() {
            let push_str = sub_resource.into_iter().collect::<Vec<_>>().join("&");
            param.push_str(&format!("&{}", push_str));
            canonicalized_resource.push_str(&format!("?{}", push_str));
            //CanonicalizedResource增加source_ip
            if let Some(source_ip) = self.source_ip {
                canonicalized_resource.push_str(&format!("&x-oss-ac-source-ip={}", source_ip));
            }
        };
        //CanonicalizedResource增加source_ip
        if let Some(source_ip) = self.source_ip {
            canonicalized_resource.push_str(&format!("&x-oss-ac-source-ip={}", source_ip));
        }
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
        param.push_str("&Signature=");
        param.push_str(&sign_str);
        //urlencode
        //let object_encode = utf8_percent_encode(&self.object.object, NON_ALPHANUMERIC).to_string();
        //let param_encode = utf8_percent_encode(&param, NON_ALPHANUMERIC).to_string();
        let url = format!(
            "https://{}.{}/{}?{}",
            self.object.bucket, self.object.client.endpoint, self.object.object, param
        );
        Url::parse(&url).ok().map(|v| v.to_string())
    }
}
