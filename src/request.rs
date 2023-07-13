use crate::{common::url_encode, Error};
use base64::{engine::general_purpose, Engine};
use chrono::{NaiveDateTime, Utc};
use hyper::{client::ResponseFuture, header, Body, Client, Method, Request};
use hyper_tls::HttpsConnector;
use ring::hmac;
use std::{borrow::Cow, collections::HashMap};

const EXCLUDED_VALUES: [&str; 84] = [
    "acl",
    "uploads",
    "location",
    "cors",
    "logging",
    "website",
    "referer",
    "lifecycle",
    "delete",
    "append",
    "tagging",
    "objectMeta",
    "uploadId",
    "partNumber",
    "security-token",
    "position",
    "img",
    "style",
    "styleName",
    "replication",
    "replicationProgress",
    "replicationLocation",
    "cname",
    "bucketInfo",
    "comp",
    "qos",
    "live",
    "status",
    "vod",
    "startTime",
    "endTime",
    "symlink",
    "x-oss-process",
    "response-content-type",
    "x-oss-traffic-limit",
    "response-content-language",
    "response-expires",
    "response-cache-control",
    "response-content-disposition",
    "response-content-encoding",
    "udf",
    "udfName",
    "udfImage",
    "udfId",
    "udfImageDesc",
    "udfApplication",
    "comp",
    "udfApplicationLog",
    "restore",
    "callback",
    "callback-var",
    "qosInfo",
    "policy",
    "stat",
    "encryption",
    "versions",
    "versioning",
    "versionId",
    "requestPayment",
    "x-oss-request-payer",
    "sequential",
    "inventory",
    "inventoryId",
    "continuation-token",
    "asyncFetch",
    "worm",
    "wormId",
    "wormExtend",
    "withHashContext",
    "x-oss-enable-md5",
    "x-oss-enable-sha1",
    "x-oss-enable-sha256",
    "x-oss-hash-ctx",
    "x-oss-md5-ctx",
    "transferAcceleration",
    "regionList",
    "cloudboxes",
    "x-oss-ac-source-ip",
    "x-oss-ac-subnet-mask",
    "x-oss-ac-vpc-id",
    "x-oss-ac-forward-allow",
    "metaQuery",
    "resourceGroup",
    "rtc",
];

//Oss基础结构
#[derive(Debug, Clone)]
pub(crate) struct Oss {
    pub ak_id: Cow<'static, str>,
    pub ak_secret: Cow<'static, str>,
    pub security_token: Option<Cow<'static, str>>,
    pub endpoint: Cow<'static, str>,
    pub bucket: Option<Cow<'static, str>>,
    pub object: Option<Cow<'static, str>>,
    pub enable_https: bool,
}
impl Oss {
    pub fn new(ak_id: &str, ak_secret: &str) -> Self {
        Oss {
            ak_id: ak_id.to_owned().into(),
            ak_secret: ak_secret.to_owned().into(),
            security_token: None,
            endpoint: "oss.aliyuncs.com".to_owned().into(),
            bucket: None,
            object: None,
            enable_https: true,
        }
    }
    pub fn set_bucket(&mut self, bucket: impl ToString) {
        self.bucket = Some(bucket.to_string().into());
    }
    pub fn set_endpoint(&mut self, endpoint: impl ToString) {
        self.endpoint = endpoint.to_string().into();
    }
    pub fn set_object(&mut self, object: impl ToString) {
        self.object = Some(object.to_string().into());
    }
    pub fn set_https(&mut self, https: bool) {
        self.enable_https = https;
    }
}
// 迭代器
#[derive(Debug)]
pub(crate) struct OssRequest {
    pub oss: Oss,
    pub method: Method,
    pub headers: HashMap<String, String>,
    pub querys: HashMap<String, String>,
    pub body: Body,
}
impl OssRequest {
    pub fn new(oss: Oss, method: Method) -> Self {
        OssRequest {
            oss,
            method,
            headers: HashMap::with_capacity(10),
            querys: HashMap::with_capacity(10),
            body: Body::empty(),
        }
    }
    pub fn set_endpoint(&mut self, endpoint: impl ToString) {
        self.oss.endpoint = endpoint.to_string().into();
    }
    pub fn set_https(&mut self, https: bool) {
        self.oss.enable_https = https;
    }
    pub fn insert_header(&mut self, key: impl ToString, value: impl ToString) {
        self.headers.insert(key.to_string(), value.to_string());
    }
    pub fn insert_query(&mut self, key: impl ToString, value: impl ToString) {
        self.querys.insert(key.to_string(), value.to_string());
    }
    pub fn set_body(&mut self, body: Body) {
        self.body = body;
    }
    pub fn uri(&self) -> String {
        //生成url
        let host = format!(
            "{}{}",
            self.oss
                .bucket
                .clone()
                .map(|v| format!("{}.", v))
                .unwrap_or_else(|| String::new()),
            self.oss.endpoint
        );
        let query = self
            .querys
            .iter()
            .map(|(key, value)| {
                let value = value.to_string();
                if value.is_empty() {
                    key.to_string()
                } else {
                    format!("{}={}", key, url_encode(&value))
                }
            })
            .collect::<Vec<_>>()
            .join("&");
        let query_str = if query.is_empty() {
            String::new()
        } else {
            format!("?{}", query)
        };
        format!(
            "https://{}/{}{}",
            host,
            url_encode(
                &self
                    .oss
                    .object
                    .clone()
                    .unwrap_or_else(|| String::new().into())
            ),
            query_str
        )
    }
    pub fn query_sign(&mut self, expires: NaiveDateTime) {
        //提取header数据
        let mut content_type = String::new();
        let mut content_md5 = String::new();
        let mut canonicalized_ossheaders = Vec::with_capacity(self.headers.len() + 1);
        self.headers.iter().for_each(|(key, value)| {
            if key.starts_with("x-oss-") {
                canonicalized_ossheaders.push(format!("{}:{}", key, value))
            };
            if key.starts_with(&header::CONTENT_TYPE.to_string()) {
                content_type = value.to_string();
            };
            if key == "Content-MD5" {
                content_md5 = value.to_string();
            };
        });
        //处理canonicalized_ossheaders
        canonicalized_ossheaders.sort();
        let mut canonicalized_ossheaders = canonicalized_ossheaders.join("\n");
        if !canonicalized_ossheaders.is_empty() {
            canonicalized_ossheaders.push_str("\n")
        }
        //构建sub_resource
        let mut sub_resource = self
            .querys
            .iter()
            .filter(|(key, _)| {
                EXCLUDED_VALUES.contains(&key.as_str()) && !(key.as_str() == "x-oss-ac-source-ip")
            })
            .map(|(key, value)| {
                if value.to_string().is_empty() {
                    key.to_owned()
                } else {
                    format!("{}={}", key, value)
                }
            })
            .collect::<Vec<_>>();
        sub_resource.sort();
        let sub_resource = sub_resource.into_iter().collect::<Vec<_>>().join("&");
        //构建canonicalized_resource
        let mut canonicalized_resource = format!(
            "/{}{}",
            self.oss
                .bucket
                .as_deref()
                .map_or(String::new(), |v| format!("{}/", v)),
            self.oss
                .object
                .as_deref()
                .map_or(String::new(), |v| format!("{}", v))
        );
        if !sub_resource.is_empty() {
            canonicalized_resource.push_str(&format!("?{}", sub_resource));
        }
        //生成待签名字符串
        let unsign_str = format!(
            "{}\n{}\n{}\n{}\n{}{}",
            self.method,
            content_md5,
            content_type,
            expires.timestamp(),
            canonicalized_ossheaders,
            canonicalized_resource
        );
        //计算签名值
        let key_str = hmac::Key::new(
            hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY,
            self.oss.ak_secret.as_bytes(),
        );
        let sign_str =
            general_purpose::STANDARD.encode(hmac::sign(&key_str, unsign_str.as_bytes()));
        self.insert_header(
            header::DATE,
            Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string(),
        );
        self.insert_query("Signature", sign_str);
        self.insert_query("OSSAccessKeyId", &self.oss.ak_id.clone());
    }
    pub fn header_sign(&mut self) {
        //提取header数据
        let mut content_type = String::new();
        let mut content_md5 = String::new();
        let mut canonicalized_ossheaders = Vec::with_capacity(self.headers.len() + 1);
        self.headers.iter().for_each(|(key, value)| {
            if key.starts_with("x-oss-") {
                canonicalized_ossheaders.push(format!("{}:{}", key, value))
            };
            if key.starts_with(&header::CONTENT_TYPE.to_string()) {
                content_type = value.to_string();
            };
            if key == "Content-MD5" {
                content_md5 = value.to_string();
            };
        });
        //处理canonicalized_ossheaders
        canonicalized_ossheaders.sort();
        let mut canonicalized_ossheaders = canonicalized_ossheaders.join("\n");
        if !canonicalized_ossheaders.is_empty() {
            canonicalized_ossheaders.push_str("\n")
        }
        //构建sub_resource
        let mut sub_resource = self
            .querys
            .iter()
            .filter(|(key, _)| EXCLUDED_VALUES.contains(&key.as_str()))
            .map(|(key, value)| {
                if value.to_string().is_empty() {
                    key.to_owned()
                } else {
                    format!("{}={}", key, value)
                }
            })
            .collect::<Vec<_>>();
        sub_resource.sort();
        let sub_resource = sub_resource.into_iter().collect::<Vec<_>>().join("&");
        //构建canonicalized_resource
        let mut canonicalized_resource = format!(
            "/{}{}",
            self.oss
                .bucket
                .as_deref()
                .map_or(String::new(), |v| format!("{}/", v)),
            self.oss
                .object
                .as_deref()
                .map_or(String::new(), |v| format!("{}", v))
        );
        if !sub_resource.is_empty() {
            canonicalized_resource.push_str(&format!("?{}", sub_resource));
        }
        //生成待签名字符串
        let date = Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string();
        let unsign_str = format!(
            "{}\n{}\n{}\n{}\n{}{}",
            self.method,
            content_md5,
            content_type,
            date,
            canonicalized_ossheaders,
            canonicalized_resource
        );
        //计算签名值
        let key_str = hmac::Key::new(
            hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY,
            self.oss.ak_secret.as_bytes(),
        );
        let sign_str =
            general_purpose::STANDARD.encode(hmac::sign(&key_str, unsign_str.as_bytes()));
        self.insert_header(header::DATE, date);
        self.insert_header(
            header::AUTHORIZATION,
            format!("OSS {}:{}", self.oss.ak_id, sign_str),
        );
    }
    pub fn send_to_oss(mut self) -> Result<ResponseFuture, Error> {
        //插入x-oss-security-token
        if let Some(security_token) = self.oss.security_token.clone() {
            self.insert_header("x-oss-security-token", security_token);
        };
        //完成签名
        self.header_sign();
        //构建http请求
        let mut req = Request::builder().method(&self.method).uri(&self.uri());
        for (key, value) in self.headers.iter() {
            req = req.header(key, value);
        }
        let request = req.body(self.body)?;
        let client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
        Ok(client.request(request))
    }
}
