// ============================================
// Invitation Codes Model - Data structure and database operations
// ============================================

use chrono::{DateTime, Duration, Utc};
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, FromRow};
/// InvitationCodes data model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InvitationCode {
    pub id: i64,
    pub user_id: Option<i64>,
    pub code: String,
    pub max_uses: i32,
    pub used_count: i32,
    pub status: String,

    // Metadata
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}

/// InvitationCodes creation request
#[derive(Debug, Deserialize)]
pub struct GenerateInvitationCodesRequest {
    pub num: usize,
}

#[derive(Debug, Serialize)]
pub struct GenerateInvitationCodesResponse {
    pub status: String,
    pub message: String,
    pub codes: Vec<String>,
    pub timestamp: u64,
}

pub struct CreateInvitationCode {
    pub code: String,
    // Metadata
    pub expires_at: DateTime<Utc>,
}


#[derive(Debug, Deserialize)]
pub struct UseInvitationCodeRequest {
    pub user_id: i64,
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct UseInvitationCodeResponse {
    pub status: String,
    pub message: String,
    pub timestamp: u64,
}

#[derive(Debug, Serialize)]
pub struct CheckUserInvitationResponse {
    pub status: String,
    pub has_active_invitation: bool,
    pub message: String,
    pub timestamp: u64,
}

#[derive(sqlx::FromRow)]
struct CountResult {
    count: i64,
}

impl InvitationCode {
    // 生成并保存邀请码
    pub async fn generate_codes(
        pool: &PgPool,
        num: usize,
    ) -> Result<Vec<String>, sqlx::Error> {
        let mut codes = Vec::with_capacity(num);
        let tx = pool.begin().await?;

        for _ in 0..num {
            let code = generate_invitation_code();
            let code1 = code.clone();

            let invitation_code = CreateInvitationCode {
                code,
                expires_at: Utc::now() + Duration::days(30),
            };

            sqlx::query_as::<_, InvitationCode>(
                r#"
            INSERT INTO invitation_codes (code, expires_at)
                VALUES ($1, $2)
            RETURNING *
            "#,
            )
                .bind(&invitation_code.code)
                .bind(&invitation_code.expires_at)
                .fetch_one(pool)
                .await?;

            codes.push(code1);
        }

        tx.commit().await?;
        Ok(codes)
    }

    // 使用邀请码
    pub async fn use_invitation_code(
        pool: &PgPool,
        user_id: i64,
        code: &str,
    ) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;

        // 先查询邀请码状态
        let invitation_code = sqlx::query_as::<_, InvitationCode>(
            r#"
            SELECT id, user_id, code, max_uses, used_count, status, expires_at, created_at, updated_at
            FROM invitation_codes
            WHERE code = $1
            "#,
        )
            .bind(code)
            .fetch_optional(&mut *tx)
            .await?;

        // println!("invitation_code: {:?}", invitation_code);

        // 检查邀请码是否存在
        let invitation_code = match invitation_code {
            Some(code) => code,
            None => {
                return Err(sqlx::Error::RowNotFound);
            }
        };

        // 检查邀请码是否已激活
        if invitation_code.status != "inactive" {
            return Err(sqlx::Error::RowNotFound); // 使用RowNotFound表示邀请码不可用
        }

        // // 检查是否已达到最大使用次数
        // if invitation_code.used_count >= invitation_code.max_uses {
        //     return Err(sqlx::Error::RowNotFound); // 使用RowNotFound表示邀请码不可用
        // }

        // 更新邀请码信息
        // let new_used_count = invitation_code.used_count + 1;
        let status = "active";

        sqlx::query_as::<_, InvitationCode>(
            r#"
            UPDATE invitation_codes
            SET user_id = $1, status = $2, updated_at = NOW()
            WHERE code = $3
            "#,)
            .bind(user_id)
            .bind(status)
            .bind(code)
            .fetch_one(pool)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    // 检查用户是否有激活的邀请码
    pub async fn check_user_has_active_invitation(
        pool: &PgPool,
        user_id: i64,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_as::<_, CountResult>(
            r#"
            SELECT COUNT(*) as count
            FROM invitation_codes
            WHERE user_id = $1 AND status = 'active'
            "#,
        ).bind(user_id)
            .fetch_one(pool)
            .await?;
        Ok(result.count > 0)
    }
}

// 生成邀请码的函数
fn generate_invitation_code() -> String {
    let random_string: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();

    format!("Alou_{}", random_string)
}
