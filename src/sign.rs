use crate::error::Error;
use base64::{engine::general_purpose, Engine};
use chrono::Utc;
use reqwest::{header, RequestBuilder};
use ring::hmac;
use std::collections::{BTreeMap, BTreeSet};

const EXCLUDED_VALUES: [&str; 38] = [
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
    "callback",
    "callback-var",
    "continuation-token",
    "stat",
    "versionId",
];
pub trait SignRequest {
    fn sign(
        self,
        ak_id: &str,
        ak_secret: &str,
        bucket: Option<&str>,
        object: Option<&str>,
    ) -> Result<RequestBuilder, Error>;
}

impl SignRequest for RequestBuilder {
    fn sign(
        self,
        ak_id: &str,
        ak_secret: &str,
        bucket: Option<&str>,
        object: Option<&str>,
    ) -> Result<RequestBuilder, Error> {
        let (client, req) = self.build_split();
        let req = req?;
        //获取Method
        let method = req.method().to_string();
        //提取header
        let headers = req.headers();
        //获取Content-MD5
        let content_md5 = headers
            .get("Content-MD5")
            .map(|value| value.to_str().unwrap_or_else(|_| ""))
            .unwrap_or_else(|| "");
        //获取Content-Type
        let content_type = headers
            .get(header::CONTENT_TYPE)
            .map(|value| value.to_str().unwrap_or_else(|_| ""))
            .unwrap_or_else(|| "");
        //准备构建CanonicalizedResource
        let mut sub_resource: BTreeSet<String> = req
            .url()
            .query_pairs()
            .filter(|(param, _)| EXCLUDED_VALUES.contains(&param.as_ref()))
            .map(|(param, value)| {
                if value.is_empty() {
                    param.to_string()
                } else {
                    format!("{}={}", param, value)
                }
            })
            .collect();
        //处理x-oss-头部
        let mut x_oss_headers = BTreeMap::new();
        for (name, value) in headers.iter() {
            // 检查头部名称是否以 "x-oss-" 开头
            if name.as_str().starts_with("x-oss-") {
                let value_str = value.to_str()?;
                x_oss_headers.insert(name.as_str().to_owned(), value_str.to_owned());
            } else if name.as_str().starts_with("response-") {
                let value_str = value.to_str()?;
                sub_resource.insert(format!("{}={}", name.as_str(), value_str));
            }
        }
        let canonicalized_ossheaders = if x_oss_headers.is_empty() {
            String::new()
        } else {
            let mut str = x_oss_headers
                .iter()
                .map(|(k, v)| format!("{}:{}", k, v))
                .collect::<Vec<_>>()
                .join("\n");
            str.push_str("\n");
            str
        };
        //正式构建CanonicalizedResource
        let mut canonicalized_resource = String::from("/");
        if let Some(bucket) = bucket {
            canonicalized_resource.push_str(&format!("{}/", bucket));
        }
        if let Some(object) = object {
            canonicalized_resource.push_str(&object.trim_matches('/'));
        }
        if !sub_resource.is_empty() {
            canonicalized_resource.push_str("?");
            canonicalized_resource
                .push_str(&sub_resource.into_iter().collect::<Vec<_>>().join("&"));
        }
        //计算签名值
        let date = Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string();
        let sign_str = format!(
            "{}\n{}\n{}\n{}\n{}{}",
            method,
            content_md5,
            content_type,
            date,
            canonicalized_ossheaders,
            canonicalized_resource
        );
        let sign_result = hmac::Key::new(hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY, ak_secret.as_bytes());
        let sign = general_purpose::STANDARD.encode(hmac::sign(&sign_result, sign_str.as_bytes()));
        //构建新的http请求
        let sign_req = reqwest::RequestBuilder::from_parts(client, req)
            .header(header::DATE, date)
            .header(header::AUTHORIZATION, format!("OSS {}:{}", ak_id, sign));
        Ok(sign_req)
    }
}
