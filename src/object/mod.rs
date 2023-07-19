//! 文件是 OSS 存储数据的基本单元，文件由元信息、用户数据和文件名（Key）组成，文件由存储空间内部唯一的Key来标识。

#[doc(hidden)]
pub use self::oss_object::OssObject;
pub use self::{
    append_object::AppendObject, copy_object::CopyObject, del_object::DelObject,
    del_object_tagging::DelObjectTagging, get_object::GetObject, get_object_acl::GetObjectAcl,
    get_object_meta::GetObjectMeta, get_object_tagging::GetObjectTagging,
    get_object_url::GetObjectUrl, get_symlink::GetSymlink, head_object::HeadObject,
    multipart_abort_upload::AbortUpload, multipart_complete_upload::CompleteUpload,
    multipart_copyto_part::CopyToPart, multipart_init_upload::InitUpload,
    multipart_list_parts::ListParts, multipart_upload_part::UploadPart, put_object::PutObject,
    put_object_acl::PutObjectAcl, put_object_tagging::PutObjectTagging, put_symlink::PutSymlink,
    restore_object::RestoreObject,
};

mod append_object;
mod copy_object;
mod del_object;
mod del_object_tagging;
mod get_object;
mod get_object_acl;
mod get_object_meta;
mod get_object_tagging;
mod get_object_url;
mod get_symlink;
mod head_object;
mod multipart_abort_upload;
mod multipart_complete_upload;
mod multipart_copyto_part;
mod multipart_init_upload;
mod multipart_list_parts;
mod multipart_upload_part;
mod oss_object;
mod put_object;
mod put_object_acl;
mod put_object_tagging;
mod put_symlink;
mod restore_object;
