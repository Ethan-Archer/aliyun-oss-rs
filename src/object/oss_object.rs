use super::{
    del_object::DelObject, AbortUpload, AppendObject, CompleteUpload, CopyObject, CopyToPart,
    DelObjectTagging, GetObject, GetObjectAcl, GetObjectMeta, GetObjectTagging, GetObjectUrl,
    GetSymlink, HeadObject, InitUpload, ListParts, PutObject, PutObjectAcl, PutObjectTagging,
    PutSymlink, RestoreObject, UploadPart,
};
use crate::{common::Acl, request::Oss};

/// OSS文件，实现了上传文件、删除文件等API
#[derive(Debug, Clone)]
pub struct OssObject {
    oss: Oss,
}

impl OssObject {
    pub(crate) fn new(mut oss: Oss, object: impl ToString) -> Self {
        oss.set_object(object);
        OssObject { oss }
    }
    /// 上传文件到OSS
    pub fn put_object(&self) -> PutObject {
        PutObject::new(self.oss.clone())
    }
    /// 追加文件
    pub fn append_object(&self) -> AppendObject {
        AppendObject::new(self.oss.clone())
    }
    /// 删除文件
    pub fn del_object(&self) -> DelObject {
        DelObject::new(self.oss.clone())
    }
    /// 获取文件访问url
    pub fn get_object_url(&self) -> GetObjectUrl {
        GetObjectUrl::new(self.oss.clone())
    }
    /// 获取文件的标签信息
    pub fn get_object_tagging(&self) -> GetObjectTagging {
        GetObjectTagging::new(self.oss.clone())
    }
    /// 获取文件完整元信息
    pub fn head_object(&self) -> HeadObject {
        HeadObject::new(self.oss.clone())
    }
    /// 获取文件的meta信息
    pub fn get_object_meta(&self) -> GetObjectMeta {
        GetObjectMeta::new(self.oss.clone())
    }
    /// 获取文件的ACL信息
    pub fn get_object_acl(&self) -> GetObjectAcl {
        GetObjectAcl::new(self.oss.clone())
    }
    /// 获取文件内容
    pub fn get_object(&self) -> GetObject {
        GetObject::new(self.oss.clone())
    }
    /// 复制文件
    pub fn copy_object(&self, copy_source: &str) -> CopyObject {
        CopyObject::new(self.oss.clone(), copy_source)
    }
    /// 解冻文件
    pub fn restore_object(&self) -> RestoreObject {
        RestoreObject::new(self.oss.clone())
    }
    /// 设置文件ACL
    pub fn put_object_acl(&self, acl: Acl) -> PutObjectAcl {
        PutObjectAcl::new(self.oss.clone(), acl)
    }
    /// 设置文件标签
    pub fn put_object_tagging(
        &self,
        tags: Vec<(impl ToString, impl ToString)>,
    ) -> PutObjectTagging {
        PutObjectTagging::new(self.oss.clone(), tags)
    }
    /// 新增软链接
    pub fn put_symlink(&self, symlink_target: impl ToString) -> PutSymlink {
        PutSymlink::new(self.oss.clone(), symlink_target)
    }
    /// 获取软链接
    pub fn get_symlink(&self) -> GetSymlink {
        GetSymlink::new(self.oss.clone())
    }
    /// 清空文件标签
    pub fn del_object_tagging(&self) -> DelObjectTagging {
        DelObjectTagging::new(self.oss.clone())
    }
    /// 初始化分片上传
    pub fn multipart_init_upload(&self) -> InitUpload {
        InitUpload::new(self.oss.clone())
    }
    /// 上传分片
    pub fn multipart_upload_part(&self, part_number: u32, upload_id: impl ToString) -> UploadPart {
        UploadPart::new(self.oss.clone(), part_number, upload_id)
    }
    /// 复制文件内容到分片
    pub fn multipart_copy_part(
        &self,
        part_number: u32,
        upload_id: impl ToString,
        copy_source: impl ToString,
    ) -> CopyToPart {
        CopyToPart::new(self.oss.clone(), part_number, upload_id, copy_source)
    }
    /// 完成分片上传
    pub fn multipart_complete_upload(&self, upload_id: impl ToString) -> CompleteUpload {
        CompleteUpload::new(self.oss.clone(), upload_id)
    }
    /// 取消分片上传
    pub fn multipart_abort_upload(&self, upload_id: impl ToString) -> AbortUpload {
        AbortUpload::new(self.oss.clone(), upload_id)
    }
    /// 列举指定Upload ID所属的所有已经上传成功Part
    pub fn multipart_list_parts(&self, upload_id: impl ToString) -> ListParts {
        ListParts::new(self.oss.clone(), upload_id)
    }
}
