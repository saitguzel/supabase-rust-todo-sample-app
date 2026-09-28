//! RBAC: RoleGuard + PermissionGuard.
use crate::errors::AppError;
use crate::middleware::auth::AuthUser;
use crate::models::profile::Role;

/// Belirli rollerden birine sahip olmayı zorunlu kılar.
/// Örn: `require_roles(&user, &[Role::Admin, Role::SuperAdmin])?;`
pub fn require_roles(user: &AuthUser, roles: &[Role]) -> Result<(), AppError> {
    if user.has_any_role(roles) {
        Ok(())
    } else {
        Err(AppError::Forbidden(format!(
            "requires one of roles: {:?}",
            roles.iter().map(|r| r.as_str()).collect::<Vec<_>>()
        )))
    }
}

/// En az belirtilen rütbeyi zorunlu kılar (hiyerarşik).
/// Örn: `require_min_role(&user, Role::Moderator)?;`
pub fn require_min_role(user: &AuthUser, min: Role) -> Result<(), AppError> {
    if user.has_min_rank(min) {
        Ok(())
    } else {
        Err(AppError::Forbidden(format!(
            "requires at least role '{}'",
            min.as_str()
        )))
    }
}

/// Granüler kaynak+aksiyon kontrolü — dinamik `role_permissions` tablosuna bakar.
/// Satır yoksa varsayılan: super_admin her şey, admin manage, diğerleri red.
pub async fn require_permission(
    db: &sqlx::PgPool,
    user: &AuthUser,
    resource: &str,
    action: &str,
) -> Result<(), AppError> {
    if user.role == Role::SuperAdmin {
        return Ok(());
    }
    let allowed: Option<bool> = sqlx::query_scalar(
        "SELECT allowed FROM role_permissions WHERE role::text = $1 AND resource = $2 AND action = $3",
    )
    .bind(user.role.as_str())
    .bind(resource)
    .bind(action)
    .fetch_optional(db)
    .await?;
    match allowed {
        Some(true) => Ok(()),
        _ => Err(AppError::Forbidden(format!(
            "missing permission {resource}:{action}"
        ))),
    }
}
