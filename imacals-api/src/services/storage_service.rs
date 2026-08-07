use bytes::Bytes;
use object_store::aws::AmazonS3Builder;
use object_store::path::Path as StorePath;
use object_store::{ObjectStore, ObjectStoreExt, PutPayload};
use crate::config::ENV;

pub struct StorageService;

impl StorageService {
    fn store() -> Result<impl ObjectStore, object_store::Error> {
        let mut builder = AmazonS3Builder::new()
            .with_bucket_name(&ENV.s3_bucket)
            .with_access_key_id(&ENV.s3_access_key)
            .with_secret_access_key(&ENV.s3_secret_key)
            .with_region(&ENV.s3_region);

        if !ENV.s3_endpoint.is_empty() {
            // MinIO (dev/staging): path-style URLs, custom endpoint, plain HTTP allowed.
            builder = builder
                .with_endpoint(&ENV.s3_endpoint)
                .with_virtual_hosted_style_request(false)
                .with_allow_http(true);
        } else {
            // AWS S3 (prod): virtual-hosted style, no custom endpoint.
            builder = builder.with_virtual_hosted_style_request(true);
        }

        builder.build()
    }

    pub async fn upload(key: &str, data: &[u8], _content_type: &str) -> Result<(), String> {
        let store = Self::store().map_err(|e| e.to_string())?;
        let path  = StorePath::from(key);
        let payload = PutPayload::from(Bytes::copy_from_slice(data));
        store.put(&path, payload).await.map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn delete(key: &str) -> Result<(), String> {
        let store = Self::store().map_err(|e| e.to_string())?;
        let path  = StorePath::from(key);
        store.delete(&path).await.map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn public_url(key: &str) -> String {
        if ENV.s3_endpoint.is_empty() {
            format!("https://{}.s3.amazonaws.com/{}", ENV.s3_bucket, key)
        } else {
            format!("{}/{}/{}", ENV.s3_endpoint, ENV.s3_bucket, key)
        }
    }
}
