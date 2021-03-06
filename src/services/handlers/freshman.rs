//! This module includes interfaces about freshman queries.
use crate::error::Result;
use crate::models::freshman::{FreshmanAnalysis, FreshmanManager, NewMate, PeopleFamiliar};
use crate::models::CommonError;
use crate::services::{response::ApiResponse, JwtToken};
use actix_web::{get, patch, post, web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;

#[derive(Debug, Deserialize)]
pub struct FreshmanReqSecret {
    pub secret: String,
}

#[get("/freshman/{account}")]
pub async fn get_basic_info(
    pool: web::Data<PgPool>,
    token: Option<JwtToken>,
    path: web::Path<String>,
    form: web::Form<FreshmanReqSecret>,
) -> Result<HttpResponse> {
    let token = token.unwrap();
    let parameters: FreshmanReqSecret = form.into_inner();
    let account = path.into_inner();
    let secret = parameters.secret;

    if account.is_empty() {
        return Err(CommonError::Parameter.into());
    }
    let manager = FreshmanManager::new(&pool);
    let freshman = manager.query(&account, secret.as_str()).await?;
    if freshman.uid.is_none() && !manager.is_bound(token.uid).await? {
        manager.bind(&freshman.student_id, Some(token.uid)).await?;
    }
    Ok(HttpResponse::Ok().json(ApiResponse::normal(freshman)))
}

#[derive(Deserialize)]
pub struct UpdateInfo {
    pub contact: Option<String>,
    pub visible: Option<bool>,
    pub secret: String,
}

#[patch("/freshman/{account}")]
pub async fn update_account(
    pool: web::Data<PgPool>,
    token: Option<JwtToken>,
    path: web::Path<String>,
    form: web::Form<UpdateInfo>,
) -> Result<HttpResponse> {
    let _ = token.unwrap();
    let account = path.into_inner();
    let form = form.into_inner();
    let secret = form.secret;

    let freshman_manager = FreshmanManager::new(&pool);
    let student = freshman_manager.query(&account, &secret).await?;

    // Set visibility.
    if let Some(visible) = form.visible {
        if visible != student.visible {
            student.set_visibility(&pool, visible).await?;
        }
    }
    // Set contact information.
    if let Some(contact) = form.contact {
        let contact_json: serde_json::Value = serde_json::from_str(contact.as_str())?;
        student.set_contact(&pool, contact_json).await?;
    }
    Ok(HttpResponse::Ok().json(&ApiResponse::empty()))
}

#[get("/freshman/{account}/roommate")]
pub async fn get_roommate(
    pool: web::Data<PgPool>,
    token: Option<JwtToken>,
    path: web::Path<String>,
    secret: web::Form<FreshmanReqSecret>,
) -> Result<HttpResponse> {
    let _ = token.unwrap();
    let account = path.into_inner();
    let secret = secret.into_inner().secret;

    #[derive(Serialize)]
    struct Resp {
        pub roommates: Vec<NewMate>,
    }

    let freshman_manager = FreshmanManager::new(&pool);
    let roommates = freshman_manager
        .query(&account, &secret)
        .await?
        .get_roommates(&pool)
        .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::normal(Resp { roommates })))
}

#[get("/freshman/{account}/familiar")]
pub async fn get_people_familiar(
    pool: web::Data<PgPool>,
    token: Option<JwtToken>,
    path: web::Path<String>,
    secret: web::Form<FreshmanReqSecret>,
) -> Result<HttpResponse> {
    let _ = token.unwrap();
    let account = path.into_inner();
    let secret = secret.into_inner().secret;

    #[derive(Serialize)]
    struct Resp {
        pub people_familiar: Vec<PeopleFamiliar>,
    }

    let freshman_manager = FreshmanManager::new(&pool);
    let people_familiar = freshman_manager
        .query(&account, &secret)
        .await?
        .get_people_familiar(&pool)
        .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::normal(Resp { people_familiar })))
}

#[get("/freshman/{account}/classmate")]
pub async fn get_classmate(
    pool: web::Data<PgPool>,
    token: Option<JwtToken>,
    path: web::Path<String>,
    secret: web::Form<FreshmanReqSecret>,
) -> Result<HttpResponse> {
    let _ = token.unwrap();
    let account = path.into_inner();
    let secret = secret.into_inner().secret;

    #[derive(Serialize)]
    struct Resp {
        pub classmates: Vec<NewMate>,
    }
    let freshman_manager = FreshmanManager::new(&pool);
    let classmates = freshman_manager
        .query(&account, &secret)
        .await?
        .get_classmates(&pool)
        .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::normal(Resp { classmates })))
}

#[get("/freshman/{account}/analysis")]
pub async fn get_analysis_data(
    pool: web::Data<PgPool>,
    token: Option<JwtToken>,
    path: web::Path<String>,
    secret: web::Form<FreshmanReqSecret>,
) -> Result<HttpResponse> {
    let _ = token.unwrap();
    let account = path.into_inner();
    let secret = secret.into_inner().secret;

    #[derive(Serialize)]
    struct Resp {
        pub freshman: FreshmanAnalysis,
    }
    let freshman_manager = FreshmanManager::new(&pool);
    let freshman = freshman_manager
        .query(&account, &secret)
        .await?
        .get_analysis(&pool)
        .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::normal(Resp { freshman })))
}

#[post("/freshman/{account}/analysis/log")]
pub async fn post_analysis_log(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    secret: web::Form<FreshmanReqSecret>,
) -> Result<HttpResponse> {
    let account = path.into_inner();
    let secret = secret.into_inner().secret;

    #[derive(Serialize)]
    struct Resp {
        pub freshman: FreshmanAnalysis,
    }
    let freshman_manager = FreshmanManager::new(&pool);
    let freshman = freshman_manager.query(&account, &secret).await?;
    sqlx::query("INSERT INTO freshman.share_log (student_id) VALUES ($1)")
        .bind(&freshman.student_id)
        .execute(pool.get_ref())
        .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::empty()))
}
