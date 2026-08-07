#[macro_export]
macro_rules! gate {
    ($pool:expr, $user:expr, $organization:expr, $permission:expr) => {
        use crate::utilities::{error_bag::ErrorBag, json_response::JsonResponse};

        if !$user.is_superuser {
            match crate::repositories::permission_repository::PermissionRepository::can($pool, &$user.id, &$organization.id, $permission).await {
                Ok(false) => return JsonResponse::error(ErrorBag::Forbidden),
                Err(e) => return JsonResponse::fatal(e, format!("{} permission check failed", $permission)),
                _ => {}
            }
        }
    };
}

#[macro_export]
macro_rules! gate_any {
    ($pool:expr, $user:expr, $organization:expr, $permissions:expr) => {
        use crate::utilities::{error_bag::ErrorBag, json_response::JsonResponse};

        if !$user.is_superuser {
            match crate::repositories::permission_repository::PermissionRepository::can_any($pool, &$user.id, &$organization.id, $permissions).await {
                Ok(false) => return JsonResponse::error(ErrorBag::Forbidden),
                Err(e) => return JsonResponse::fatal(e, "canAny permission check failed"),
                _ => {}
            }
        }
    };
}

#[macro_export]
macro_rules! gate_all {
    ($pool:expr, $user:expr, $organization:expr, $permissions:expr) => {
        use crate::utilities::{error_bag::ErrorBag, json_response::JsonResponse};

        if !$user.is_superuser {
            match crate::repositories::permission_repository::PermissionRepository::can_all($pool, &$user.id, &$organization.id, $permissions).await {
                Ok(false) => return JsonResponse::error(ErrorBag::Forbidden),
                Err(e) => return JsonResponse::fatal(e, "canAll permission check failed"),
                _ => {}
            }
        }
    };
}


#[macro_export]
macro_rules! can {
    ($pool:expr, $user:expr, $organization:expr, $permission:expr) => {
        if $user.is_superuser {
            true
        } else {
            crate::repositories::permission_repository::PermissionRepository::can($pool, &$user.id, &$organization.id, $permission).await.unwrap_or(false)
        }
    };
}

#[macro_export]
macro_rules! can_any {
    ($pool:expr, $user:expr, $organization:expr, $permissions:expr) => {
        if $user.is_superuser {
            true
        } else {
            crate::repositories::permission_repository::PermissionRepository::can_any($pool, &$user.id, &$organization.id, $permissions).await.unwrap_or(false)
        }
    };
}

#[macro_export]
macro_rules! can_all {
    ($pool:expr, $user:expr, $organization:expr, $permissions:expr) => {
        if $user.is_superuser {
            return true
        } else {
            return crate::repositories::permission_repository::PermissionRepository::can_all($pool, &$user.id, &$organization.id, $permissions).await.unwrap_or(false)
        }
    };
}