use actix_multipart::Multipart;
use actix_web::web::{Data, Path};
use actix_web::HttpResponse;
use futures_util::StreamExt as _;
use serde_json::json;
use sqlx::Error;
use uuid::Uuid;

use crate::AppState;
use crate::models::file::{CreateFileInput, FileType};
use crate::models::organization::Organization;
use crate::models::user::User;
use crate::repositories::file_repository::FileRepository;
use crate::repositories::organization_user_role_repository::OrganizationUserRoleRepository;
use crate::services::storage_service::StorageService;
use crate::utilities::error_bag::ErrorBag;
use crate::utilities::json_response::JsonResponse;

/// Valid document_type values accepted from the client.
const DOCUMENT_TYPES: &[&str] = &["signature", "initials", "proof_of_funds"];

/// proof_of_funds uploads are restricted to project-manager role.
const PROOF_OF_FUNDS_ROLE: &str = "project-manager";

fn to_file_type(document_type: &str) -> FileType {
    match document_type {
        "initials"       => FileType::UserInitials,
        "proof_of_funds" => FileType::UserProofOfFunds,
        _                => FileType::UserSignature,
    }
}

pub async fn index(user: User, organization: Organization, app: Data<AppState>, id: Path<Uuid>) -> HttpResponse {
    crate::gate!(&app.pool, &user, &organization, "users.view");
    let user_id = id.into_inner();
    match FileRepository::find_for_owner(&app.pool, "users", &user_id).await {
        Ok(files) => JsonResponse::success(files),
        Err(e)    => JsonResponse::fatal(e, "user_document_controller.index failed"),
    }
}

pub async fn create(
    user: User,
    organization: Organization,
    app: Data<AppState>,
    id: Path<Uuid>,
    mut payload: Multipart,
) -> HttpResponse {
    crate::gate!(&app.pool, &user, &organization, "users.update");

    let target_user_id = id.into_inner();
    let mut document_type = String::new();
    let mut file_bytes: Vec<u8> = Vec::new();
    let mut file_name = String::new();
    let mut mime_type = String::from("application/octet-stream");

    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(f) => f,
            Err(e) => return HttpResponse::BadRequest().json(json!({ "error": e.to_string() })),
        };

        let disposition = field.content_disposition().cloned();
        let name = disposition.as_ref().and_then(|d| d.get_name()).unwrap_or("").to_string();

        match name.as_str() {
            "document_type" => {
                while let Some(chunk) = field.next().await {
                    match chunk {
                        Ok(b)  => document_type.push_str(&String::from_utf8_lossy(&b)),
                        Err(e) => return HttpResponse::BadRequest().json(json!({ "error": e.to_string() })),
                    }
                }
            }
            "file" => {
                file_name = disposition.as_ref()
                    .and_then(|d| d.get_filename())
                    .unwrap_or("upload")
                    .to_string();
                if let Some(ct) = field.content_type() {
                    mime_type = ct.to_string();
                }
                while let Some(chunk) = field.next().await {
                    match chunk {
                        Ok(b)  => file_bytes.extend_from_slice(&b),
                        Err(e) => return HttpResponse::BadRequest().json(json!({ "error": e.to_string() })),
                    }
                }
            }
            _ => {}
        }
    }

    let document_type = document_type.trim().to_lowercase();

    if document_type.is_empty() {
        return JsonResponse::error(ErrorBag::Validation {
            field: "document_type".into(),
            message: "document_type is required".into(),
        });
    }
    if !DOCUMENT_TYPES.contains(&document_type.as_str()) {
        return JsonResponse::error(ErrorBag::Validation {
            field: "document_type".into(),
            message: format!("Must be one of: {}", DOCUMENT_TYPES.join(", ")),
        });
    }
    if file_bytes.is_empty() {
        return JsonResponse::error(ErrorBag::Validation {
            field: "file".into(),
            message: "file is required".into(),
        });
    }

    // proof_of_funds is restricted to project-manager role only.
    if document_type == "proof_of_funds" {
        let role = OrganizationUserRoleRepository::get_user_role_for_user(
            &app.pool, &target_user_id, &organization.id,
        ).await;
        let is_pm = matches!(role, Ok(Some(ref r)) if r.name == PROOF_OF_FUNDS_ROLE);
        if !is_pm {
            return JsonResponse::error(ErrorBag::Forbidden);
        }
    }

    let ext = file_name.rsplit('.').next().unwrap_or("bin").to_lowercase();
    let relative_path = format!("users/{}/{}/{}.{}", target_user_id, document_type, Uuid::new_v4(), ext);
    let absolute_path = StorageService::public_url(&relative_path);

    if let Err(e) = StorageService::upload(&relative_path, &file_bytes, &mime_type).await {
        return JsonResponse::fatal(
            Error::Protocol(e),
            "user_document_controller.create.upload failed",
        );
    }

    let input = CreateFileInput {
        created_by:    user.id,
        fileable_type: "users".into(),
        fileable_id:   target_user_id,
        file_type:     to_file_type(&document_type),
        name:          file_name,
        absolute_path,
        relative_path,
        size:          file_bytes.len() as i64,
        mime_type,
    };

    match FileRepository::create(&app.pool, &input).await {
        Ok(file) => JsonResponse::success(file),
        Err(e)   => JsonResponse::fatal(e, "user_document_controller.create failed"),
    }
}

pub async fn delete(
    user: User,
    organization: Organization,
    app: Data<AppState>,
    path: Path<(Uuid, Uuid)>,
) -> HttpResponse {
    crate::gate!(&app.pool, &user, &organization, "users.update");
    let (user_id, file_id) = path.into_inner();
    match FileRepository::delete_for_owner(&app.pool, &file_id, "users", &user_id).await {
        Ok(0) => JsonResponse::error(ErrorBag::NotFound("Document".into())),
        Ok(_) => JsonResponse::success(json!({ "message": "Document deleted successfully" })),
        Err(e) => JsonResponse::fatal(e, "user_document_controller.delete failed"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;
    use actix_web::{test, web, App};
    use sqlx::PgPool;
    use crate::AppState;
    use crate::services::jwt_service::JwtService;

    // ── helpers ────────────────────────────────────────────────────────────────

    async fn make_user(pool: &PgPool, email: &str, superuser: bool) -> (Uuid, String) {
        let id = sqlx::query_scalar!(
            "INSERT INTO users (first_name, last_name, email, password, is_superuser, current_logged_in_at)
             VALUES ('T','T',$1,'x',$2,NOW()) RETURNING id",
            email, superuser
        ).fetch_one(pool).await.unwrap();
        let token = format!("Bearer {}", JwtService::create_access_token(id, 60).unwrap());
        (id, token)
    }

    // created_by is NOT NULL on organizations, so the owning user has to be supplied.
    async fn make_org(pool: &PgPool, created_by: &Uuid) -> Uuid {
        sqlx::query_scalar!(
            "INSERT INTO organizations (name, slug, created_by) VALUES ('Test Org', 'test-org', $1) RETURNING id",
            created_by
        ).fetch_one(pool).await.unwrap()
    }

    // Returns App<T> rather than `impl ServiceFactory<..>`: App itself is only an
    // IntoServiceFactory, so the latter never satisfies test::init_service.
    fn app_scope(pool: PgPool) -> App<
        impl actix_web::dev::ServiceFactory<
            actix_web::dev::ServiceRequest,
            Config = (),
            Response = actix_web::dev::ServiceResponse,
            Error = actix_web::Error,
            InitError = (),
        >,
    > {
        App::new()
            .app_data(web::Data::new(AppState { pool }))
            .service(
                web::scope("/users")
                    .route("/{id}/documents",          web::get().to(index))
                    .route("/{id}/documents",          web::post().to(create))
                    .route("/{id}/documents/{doc_id}", web::delete().to(delete)),
            )
    }

    /// Builds a minimal `multipart/form-data` body.
    /// Each entry: `(field_name, filename_or_empty, content_type_or_none, data)`.
    fn multipart_body(boundary: &str, fields: &[(&str, &str, Option<&str>, &[u8])]) -> Vec<u8> {
        let mut body = Vec::new();
        for (name, filename, ct, data) in fields {
            body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
            if filename.is_empty() {
                body.extend_from_slice(
                    format!("Content-Disposition: form-data; name=\"{name}\"\r\n").as_bytes(),
                );
            } else {
                body.extend_from_slice(
                    format!("Content-Disposition: form-data; name=\"{name}\"; filename=\"{filename}\"\r\n")
                        .as_bytes(),
                );
            }
            if let Some(c) = ct {
                body.extend_from_slice(format!("Content-Type: {c}\r\n").as_bytes());
            }
            body.extend_from_slice(b"\r\n");
            body.extend_from_slice(data);
            body.extend_from_slice(b"\r\n");
        }
        body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
        body
    }

    // ── auth & organisation ────────────────────────────────────────────────────

    #[sqlx::test(migrations = "./src/migrations")]
    async fn unauthenticated_index_returns_401(pool: PgPool) {
        let user_id = make_user(&pool, "anon@test.com", false).await.0;
        let app = test::init_service(app_scope(pool)).await;
        let req = test::TestRequest::get()
            .uri(&format!("/users/{user_id}/documents"))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::UNAUTHORIZED);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn missing_org_header_returns_401(pool: PgPool) {
        let (user_id, token) = make_user(&pool, "noorg@test.com", false).await;
        let app = test::init_service(app_scope(pool)).await;
        let req = test::TestRequest::get()
            .uri(&format!("/users/{user_id}/documents"))
            .insert_header(("Authorization", token))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::UNAUTHORIZED);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn user_without_permission_cannot_list(pool: PgPool) {
        let (user_id, token) = make_user(&pool, "noperm@test.com", false).await;
        let org_id = make_org(&pool, &user_id).await;
        let app = test::init_service(app_scope(pool)).await;
        let req = test::TestRequest::get()
            .uri(&format!("/users/{user_id}/documents"))
            .insert_header(("Authorization", token))
            .insert_header(("X-Organization-Id", org_id.to_string()))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::FORBIDDEN);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn user_without_permission_cannot_upload(pool: PgPool) {
        let boundary = "testboundary";
        let (user_id, token) = make_user(&pool, "noupload@test.com", false).await;
        let org_id = make_org(&pool, &user_id).await;
        let body = multipart_body(boundary, &[
            ("document_type", "", None, b"signature"),
            ("file", "sig.png", Some("image/png"), b"\x89PNG"),
        ]);
        let app = test::init_service(app_scope(pool)).await;
        let req = test::TestRequest::post()
            .uri(&format!("/users/{user_id}/documents"))
            .insert_header(("Authorization", token))
            .insert_header(("X-Organization-Id", org_id.to_string()))
            .insert_header(("Content-Type", format!("multipart/form-data; boundary={boundary}")))
            .set_payload(body)
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::FORBIDDEN);
    }

    // ── index ──────────────────────────────────────────────────────────────────

    #[sqlx::test(migrations = "./src/migrations")]
    async fn superuser_can_list_documents_returns_empty(pool: PgPool) {
        let (user_id, token) = make_user(&pool, "superlist@test.com", true).await;
        let org_id = make_org(&pool, &user_id).await;
        let app = test::init_service(app_scope(pool)).await;
        let req = test::TestRequest::get()
            .uri(&format!("/users/{user_id}/documents"))
            .insert_header(("Authorization", token))
            .insert_header(("X-Organization-Id", org_id.to_string()))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["data"].as_array().unwrap().is_empty());
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn index_returns_existing_file_for_user(pool: PgPool) {
        let (user_id, token) = make_user(&pool, "indexfile@test.com", true).await;
        let org_id = make_org(&pool, &user_id).await;

        sqlx::query!(
            r#"INSERT INTO files
               (created_by, fileable_type, fileable_id, "type", name,
                absolute_path, relative_path, size, mime_type)
               VALUES ($1, 'users', $1, 'user-signature', 'sig.png',
                       'http://minio/sig.png', 'users/sig.png', 512, 'image/png')"#,
            user_id
        ).execute(&pool).await.unwrap();

        let app = test::init_service(app_scope(pool)).await;
        let req = test::TestRequest::get()
            .uri(&format!("/users/{user_id}/documents"))
            .insert_header(("Authorization", token))
            .insert_header(("X-Organization-Id", org_id.to_string()))
            .to_request();
        let body: serde_json::Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(body["data"].as_array().unwrap().len(), 1);
        assert_eq!(body["data"][0]["name"], "sig.png");
    }

    // ── create validation ──────────────────────────────────────────────────────

    #[sqlx::test(migrations = "./src/migrations")]
    async fn upload_rejects_missing_document_type(pool: PgPool) {
        let boundary = "bound1";
        let (user_id, token) = make_user(&pool, "nodoctype@test.com", true).await;
        let org_id = make_org(&pool, &user_id).await;
        // Only the file field — no document_type.
        let body = multipart_body(boundary, &[
            ("file", "test.png", Some("image/png"), b"\x89PNG"),
        ]);
        let app = test::init_service(app_scope(pool)).await;
        let req = test::TestRequest::post()
            .uri(&format!("/users/{user_id}/documents"))
            .insert_header(("Authorization", token))
            .insert_header(("X-Organization-Id", org_id.to_string()))
            .insert_header(("Content-Type", format!("multipart/form-data; boundary={boundary}")))
            .set_payload(body)
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn upload_rejects_invalid_document_type(pool: PgPool) {
        let boundary = "bound2";
        let (user_id, token) = make_user(&pool, "baddoctype@test.com", true).await;
        let org_id = make_org(&pool, &user_id).await;
        let body = multipart_body(boundary, &[
            ("document_type", "", None, b"invoice"),
            ("file", "test.png", Some("image/png"), b"\x89PNG"),
        ]);
        let app = test::init_service(app_scope(pool)).await;
        let req = test::TestRequest::post()
            .uri(&format!("/users/{user_id}/documents"))
            .insert_header(("Authorization", token))
            .insert_header(("X-Organization-Id", org_id.to_string()))
            .insert_header(("Content-Type", format!("multipart/form-data; boundary={boundary}")))
            .set_payload(body)
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn upload_rejects_missing_file_bytes(pool: PgPool) {
        let boundary = "bound3";
        let (user_id, token) = make_user(&pool, "nofile@test.com", true).await;
        let org_id = make_org(&pool, &user_id).await;
        // document_type present but no file field at all.
        let body = multipart_body(boundary, &[
            ("document_type", "", None, b"signature"),
        ]);
        let app = test::init_service(app_scope(pool)).await;
        let req = test::TestRequest::post()
            .uri(&format!("/users/{user_id}/documents"))
            .insert_header(("Authorization", token))
            .insert_header(("X-Organization-Id", org_id.to_string()))
            .insert_header(("Content-Type", format!("multipart/form-data; boundary={boundary}")))
            .set_payload(body)
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    // ── delete ─────────────────────────────────────────────────────────────────

    #[sqlx::test(migrations = "./src/migrations")]
    async fn delete_nonexistent_document_returns_404(pool: PgPool) {
        let (user_id, token) = make_user(&pool, "del404@test.com", true).await;
        let org_id = make_org(&pool, &user_id).await;
        let ghost_id = Uuid::new_v4();
        let app = test::init_service(app_scope(pool)).await;
        let req = test::TestRequest::delete()
            .uri(&format!("/users/{user_id}/documents/{ghost_id}"))
            .insert_header(("Authorization", token))
            .insert_header(("X-Organization-Id", org_id.to_string()))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn superuser_can_delete_existing_document(pool: PgPool) {
        let (user_id, token) = make_user(&pool, "delsuccess@test.com", true).await;
        let org_id = make_org(&pool, &user_id).await;

        let file_id = sqlx::query_scalar!(
            r#"INSERT INTO files
               (created_by, fileable_type, fileable_id, "type", name,
                absolute_path, relative_path, size, mime_type)
               VALUES ($1, 'users', $1, 'user-signature', 'sig.png',
                       'http://minio/sig.png', 'users/sig.png', 512, 'image/png')
               RETURNING id"#,
            user_id
        ).fetch_one(&pool).await.unwrap();

        let app = test::init_service(app_scope(pool)).await;
        let req = test::TestRequest::delete()
            .uri(&format!("/users/{user_id}/documents/{file_id}"))
            .insert_header(("Authorization", token))
            .insert_header(("X-Organization-Id", org_id.to_string()))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::OK);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn delete_cannot_cross_user_boundary(pool: PgPool) {
        // File belongs to user_a; user_b (even as superuser) should get 404
        // because delete_for_owner scopes by fileable_id.
        let (user_a_id, _)      = make_user(&pool, "owner@test.com",   true).await;
        let (user_b_id, token_b) = make_user(&pool, "attacker@test.com", true).await;
        let org_id = make_org(&pool, &user_a_id).await;

        let file_id = sqlx::query_scalar!(
            r#"INSERT INTO files
               (created_by, fileable_type, fileable_id, "type", name,
                absolute_path, relative_path, size, mime_type)
               VALUES ($1, 'users', $1, 'user-initials', 'init.png',
                       'http://minio/init.png', 'users/init.png', 128, 'image/png')
               RETURNING id"#,
            user_a_id
        ).fetch_one(&pool).await.unwrap();

        let app = test::init_service(app_scope(pool)).await;
        // Request is made against user_b's URL, but file belongs to user_a.
        let req = test::TestRequest::delete()
            .uri(&format!("/users/{user_b_id}/documents/{file_id}"))
            .insert_header(("Authorization", token_b))
            .insert_header(("X-Organization-Id", org_id.to_string()))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::NOT_FOUND);
    }
}
