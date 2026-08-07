use actix_web::web::{Data, Json, Path};
use actix_web::HttpResponse;
use serde_json::json;
use sqlx::Error;
use uuid::Uuid;

use crate::config::ENV;
use crate::AppState;
use crate::models::attribute::{CreateAttributeSchema, UpdateAttributeSchema};
use crate::models::user::User;
use crate::repositories::attribute_repository::AttributeRepository;
use crate::utilities::encryption;
use crate::utilities::error_bag::ErrorBag;
use crate::utilities::json_response::JsonResponse;

// Attributes may hold plaintext or encrypted credentials — writes are superuser-only.
macro_rules! require_superuser {
    ($user:expr) => {
        if !$user.is_superuser {
            return JsonResponse::error(ErrorBag::Forbidden);
        }
    };
}

pub async fn create(
    user: User,
    app: Data<AppState>,
    mut body: Json<CreateAttributeSchema>,
) -> HttpResponse {
    require_superuser!(user);
    // Encrypt the value before storing if the caller flagged it as sensitive.
    if body.is_encrypted == Some(true) {
        if let Some(ref plaintext) = body.value.clone() {
            match encryption::encrypt(plaintext, &ENV.app_secret) {
                Ok(ciphertext) => body.value = Some(ciphertext),
                Err(e) => return JsonResponse::error(ErrorBag::InternalServerError(
                    format!("Failed to encrypt attribute value: {e}")
                )),
            }
        }
    }
    match AttributeRepository::create(&app.pool, &user.id, &body).await {
        Ok(attr) => JsonResponse::success(attr),
        Err(e)   => JsonResponse::fatal(e, "attribute_controller.create failed"),
    }
}

pub async fn update(
    user: User,
    app: Data<AppState>,
    id: Path<Uuid>,
    mut body: Json<UpdateAttributeSchema>,
) -> HttpResponse {
    require_superuser!(user);
    let id = id.into_inner();

    // Encrypt the new value when the attribute is flagged as encrypted.
    // When is_encrypted is absent from the patch body, inherit it from the stored record.
    if body.value.is_some() {
        let should_encrypt = match body.is_encrypted {
            Some(flag) => flag,
            None => match AttributeRepository::find_by_id(&app.pool, &id).await {
                Ok(existing)            => existing.is_encrypted,
                Err(Error::RowNotFound) => return JsonResponse::error(ErrorBag::NotFound("Attribute".into())),
                Err(e)                  => return JsonResponse::fatal(e, "attribute_controller.update failed"),
            },
        };

        if should_encrypt {
            let plaintext = body.value.clone().unwrap();
            match encryption::encrypt(&plaintext, &ENV.app_secret) {
                Ok(ciphertext) => body.value = Some(ciphertext),
                Err(e)         => return JsonResponse::error(ErrorBag::InternalServerError(
                    format!("Failed to encrypt attribute value: {e}")
                )),
            }
        }
    }

    match AttributeRepository::update(&app.pool, &id, &body).await {
        Ok(attr)                => JsonResponse::success(attr),
        Err(Error::RowNotFound) => JsonResponse::error(ErrorBag::NotFound("Attribute".into())),
        Err(e)                  => JsonResponse::fatal(e, "attribute_controller.update failed"),
    }
}

pub async fn delete(user: User, app: Data<AppState>, id: Path<Uuid>) -> HttpResponse {
    require_superuser!(user);
    match AttributeRepository::delete(&app.pool, &id.into_inner()).await {
        Ok(0) => JsonResponse::error(ErrorBag::NotFound("Attribute".into())),
        Ok(_) => JsonResponse::success(json!({ "message": "Attribute deleted successfully" })),
        Err(e) => JsonResponse::fatal(e, "attribute_controller.delete failed"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;
    use actix_web::{test, web, App};
    use serde_json::json;
    use sqlx::PgPool;

    use crate::services::jwt_service::JwtService;
    use crate::AppState;

    async fn make_token(pool: &PgPool, email: &str, superuser: bool) -> String {
        let id = sqlx::query_scalar!(
            "INSERT INTO users (first_name, last_name, email, password, is_superuser, current_logged_in_at)
             VALUES ('T','T',$1,'x',$2,NOW()) RETURNING id",
            email,
            superuser
        )
        .fetch_one(pool)
        .await
        .unwrap();
        format!("Bearer {}", JwtService::create_access_token(id, 60).unwrap())
    }

    fn attr_payload(owner_id: Uuid) -> serde_json::Value {
        json!({
            "attributeable_type": "integrations",
            "attributeable_id":   owner_id,
            "name":               "url",
            "value":              "https://rets.example.com",
            "type":               "url",
            "is_encrypted":       false,
        })
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn regular_user_cannot_create_attribute(pool: PgPool) {
        let token = make_token(&pool, "regular_attr@test.com", false).await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { pool: pool.clone() }))
                .service(web::scope("/attributes").route("", web::post().to(create))),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/attributes")
            .insert_header(("Authorization", token))
            .set_json(attr_payload(Uuid::new_v4()))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::FORBIDDEN);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn superuser_can_create_attribute(pool: PgPool) {
        let token = make_token(&pool, "admin_attr_create@test.com", true).await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { pool: pool.clone() }))
                .service(web::scope("/attributes").route("", web::post().to(create))),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/attributes")
            .insert_header(("Authorization", token))
            .set_json(attr_payload(Uuid::new_v4()))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::OK);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn superuser_can_update_attribute(pool: PgPool) {
        let user_id = sqlx::query_scalar!(
            "INSERT INTO users (first_name, last_name, email, password, is_superuser, current_logged_in_at)
             VALUES ('T','T','admin_attr_upd@test.com','x',true,NOW()) RETURNING id"
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        let token = format!("Bearer {}", JwtService::create_access_token(user_id, 60).unwrap());
        let attr_id = sqlx::query_scalar!(
            "INSERT INTO attributes (created_by, attributeable_type, attributeable_id, name, value, type, is_encrypted)
             VALUES ($1, 'integrations', $2, 'url', 'https://old.example.com', 'url', false)
             RETURNING id",
            user_id,
            Uuid::new_v4(),
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { pool: pool.clone() }))
                .service(web::scope("/attributes").route("/{id}", web::put().to(update))),
        )
        .await;
        let req = test::TestRequest::put()
            .uri(&format!("/attributes/{}", attr_id))
            .insert_header(("Authorization", token))
            .set_json(json!({ "value": "https://new.example.com" }))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::OK);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn update_nonexistent_attribute_returns_404(pool: PgPool) {
        let token = make_token(&pool, "admin_attr_upd2@test.com", true).await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { pool: pool.clone() }))
                .service(web::scope("/attributes").route("/{id}", web::put().to(update))),
        )
        .await;
        let req = test::TestRequest::put()
            .uri(&format!("/attributes/{}", Uuid::new_v4()))
            .insert_header(("Authorization", token))
            .set_json(json!({ "value": "new-value" }))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn superuser_can_delete_attribute(pool: PgPool) {
        let user_id = sqlx::query_scalar!(
            "INSERT INTO users (first_name, last_name, email, password, is_superuser, current_logged_in_at)
             VALUES ('T','T','admin_attr_del@test.com','x',true,NOW()) RETURNING id"
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        let token = format!("Bearer {}", JwtService::create_access_token(user_id, 60).unwrap());
        let attr_id = sqlx::query_scalar!(
            "INSERT INTO attributes (created_by, attributeable_type, attributeable_id, name, value, type, is_encrypted)
             VALUES ($1, 'integrations', $2, 'url', 'https://del.example.com', 'url', false)
             RETURNING id",
            user_id,
            Uuid::new_v4(),
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { pool: pool.clone() }))
                .service(web::scope("/attributes").route("/{id}", web::delete().to(delete))),
        )
        .await;
        let req = test::TestRequest::delete()
            .uri(&format!("/attributes/{}", attr_id))
            .insert_header(("Authorization", token))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::OK);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn delete_nonexistent_attribute_returns_404(pool: PgPool) {
        let token = make_token(&pool, "admin_attr_del2@test.com", true).await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { pool: pool.clone() }))
                .service(web::scope("/attributes").route("/{id}", web::delete().to(delete))),
        )
        .await;
        let req = test::TestRequest::delete()
            .uri(&format!("/attributes/{}", Uuid::new_v4()))
            .insert_header(("Authorization", token))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn regular_user_cannot_delete_attribute(pool: PgPool) {
        let token = make_token(&pool, "regular_attr_del@test.com", false).await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { pool: pool.clone() }))
                .service(web::scope("/attributes").route("/{id}", web::delete().to(delete))),
        )
        .await;
        let req = test::TestRequest::delete()
            .uri(&format!("/attributes/{}", Uuid::new_v4()))
            .insert_header(("Authorization", token))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::FORBIDDEN);
    }
}
