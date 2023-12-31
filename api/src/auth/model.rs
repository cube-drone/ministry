use std::collections::HashMap;
use std::sync::Arc;
use std::env;
use std::net::IpAddr;

use serde::{Deserialize, Serialize};

use redis::AsyncCommands;

use anyhow::Result;
use anyhow::anyhow;
use rocket::tokio::task;

use rocket::serde::uuid::Uuid;
use scylla::prepared_statement::PreparedStatement;
use scylla::macros::FromRow;
//use scylla::frame::value::Timestamp;
use scylla::Session;
use chrono::{Utc, Duration};

// hashing some stuff
use ::argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use std::io::Cursor;
use murmur3::murmur3_x86_128;

use crate::email::EmailAddress;
use crate::Services;

const ROOT_USER_ID: UserId = UserId(Uuid::from_u128(0));
const DEFAULT_THUMBNAIL_URL: &str = "/static/chismas.png";

const USER_SESSION_TIMEOUT_SECONDS: usize = 86400 * 14; // two weeks
const EMAIL_VERIFICATION_TIMEOUT_SECONDS: usize = 86400 * 3; // three days
const PASSWORD_RESET_TIMEOUT_SECONDS: usize = 86400 * 1; // one day
const USER_MAX_SESSION_COUNT: usize = 8; // how many sessions can a single user have active?

pub async fn initialize(
    scylla_session: &Arc<Session>,
) -> Result<HashMap<&'static str, PreparedStatement>> {

    let mut prepared_queries = HashMap::new();
    scylla_session
        .query(r#"
            CREATE TABLE IF NOT EXISTS ks.user (
                id uuid PRIMARY KEY,
                display_name text,
                parent_id uuid,
                hashed_password text,
                thumbnail_url text,
                email text,
                is_verified boolean,
                is_admin boolean,
                tags set<text>,
                created_at timestamp,
                updated_at timestamp);
        "#, &[], ).await?;

        prepared_queries.insert(
            "create_user",
            scylla_session
                .prepare("INSERT INTO ks.user (id, display_name, parent_id, hashed_password, email, thumbnail_url, is_verified, is_admin, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?);")
                .await?,
        );

        prepared_queries.insert(
            "get_user_exists",
            scylla_session
                .prepare("SELECT id FROM ks.user WHERE id = ?;")
                .await?,
        );

        prepared_queries.insert(
            "get_user",
            scylla_session
                .prepare("SELECT id, display_name, parent_id, hashed_password, email, thumbnail_url, is_verified, is_admin, tags, created_at, updated_at FROM ks.user WHERE id = ?;")
                .await?,
        );

        prepared_queries.insert(
            "delete_user",
            scylla_session
                .prepare("DELETE FROM ks.user WHERE id = ?;")
                .await?,
        );

        prepared_queries.insert(
            "verify_user_email",
            scylla_session
                .prepare("UPDATE ks.user SET is_verified = true WHERE id = ?;")
                .await?,
        );

        prepared_queries.insert(
            "change_user_password",
            scylla_session
                .prepare("UPDATE ks.user SET hashed_password = ? WHERE id = ?;")
                .await?,
        );

    scylla_session
        .query(r#"
            CREATE TABLE IF NOT EXISTS ks.user_invite (
                user_id uuid PRIMARY KEY,
                invite_key uuid,
                created_at timestamp );
            "#, &[], ).await?;

    scylla_session
        .query(r#"
            CREATE TABLE IF NOT EXISTS ks.user_parents (
                user_id uuid PRIMARY KEY,
                parents list<uuid> );
            "#, &[], ).await?;

    scylla_session
        .query(r#"
            CREATE TABLE IF NOT EXISTS ks.user_children (
                user_id uuid,
                child_id uuid,
                PRIMARY KEY (user_id, child_id));
            "#, &[], ).await?;

    scylla_session
        .query(r#"
            CREATE TABLE IF NOT EXISTS ks.user_descendants (
                user_id uuid,
                descendant_id uuid,
                PRIMARY KEY (user_id, descendant_id));
            "#, &[], ).await?;

    // user --> ip
    scylla_session
        .query(r#"
            CREATE TABLE IF NOT EXISTS ks.user_ips (
                user_id uuid,
                ip inet,
                PRIMARY KEY(user_id, ip));
            "#, &[], ).await?;

        /* we don't have a plan for this one yet: get all of the ips for a given user */
        prepared_queries.insert(
            "get_user_ips",
            scylla_session
                .prepare("SELECT ip FROM ks.user_ips WHERE user_id = ?;")
                .await?,
        );

        // register an ip against a user
        // this lasts forever: if you've _ever_ logged in from an IP, it's good forever
        prepared_queries.insert(
            "set_user_ip",
            scylla_session
                .prepare("INSERT INTO ks.user_ips (user_id, ip) VALUES (?, ?);")
                .await?,
        );

        prepared_queries.insert(
            "delete_user_ip",
            scylla_session
                .prepare("DELETE FROM ks.user_ips WHERE user_id = ? AND ip = ?;")
                .await?,
        );

        // this one's mostly here to test whether or not any given ip is "known" to us
        // if not, we need to send a verification email
        prepared_queries.insert(
            "get_user_ip",
            scylla_session
                .prepare("SELECT ip FROM ks.user_ips WHERE user_id = ? AND ip = ?;")
                .await?,
        );

    // email --> user
    scylla_session
        .query(r#"
            CREATE TABLE IF NOT EXISTS ks.email_user (
                email text PRIMARY KEY,
                user_id uuid)
            "#, &[], ).await?;

        prepared_queries.insert(
            "get_email_user",
            scylla_session
                .prepare("SELECT user_id FROM ks.email_user WHERE email = ?;")
                .await?,
        );

        prepared_queries.insert(
            "set_email_user",
            scylla_session
                .prepare("INSERT INTO ks.email_user (email, user_id) VALUES (?, ?);")
                .await?,
        );

    // email_domain --> user
    scylla_session
        .query(r#"
            CREATE TABLE IF NOT EXISTS ks.email_domain (
                email_domain text,
                user_id uuid,
                PRIMARY KEY (email_domain, user_id))
            "#, &[], ).await?;

    /*
    prepared_queries.insert(
        "create_user_invite",
        scylla_session
            .prepare("INSERT INTO ks.user_invite (user_id, invite_key, uses_remaining, created_at, updated_at) VALUES (?, ?, ?, ?, ?);")
            .await?,
    );

    prepared_queries.insert(
        "update_user_password",
        scylla_session
            .prepare("UPDATE ks.user USING TTL 0 SET hashed_password = ? WHERE id = ?;")
            .await?,
    );

    */

    Ok(prepared_queries)
}

pub fn password_hash(password: &str) -> Result<String> {
    let peppered: String = format!("{}-{}-{}", password, env::var("GROOVELET_PEPPER").unwrap_or_else(|_| "peppa".to_string()), "SPUDJIBMSPLQPFFSPLBLBlBLBLPRT");
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hashed_password = argon2.hash_password(peppered.as_bytes(), &salt).expect("strings should be hashable").to_string();
    Ok(hashed_password)
}

pub async fn password_hash_async(password: &str) -> Result<String> {
    let password = password.to_string();
    let result = task::spawn_blocking(move || {
        password_hash(&password)
    }).await?;

    result
}

pub fn password_test(password: &str, hashed_password: &str) -> Result<bool> {
    let peppered: String = format!("{}-{}-{}", password, env::var("GROOVELET_PEPPER").unwrap_or_else(|_| "peppa".to_string()), "SPUDJIBMSPLQPFFSPLBLBlBLBLPRT");
    let argon2 = Argon2::default();
    let password_hash = PasswordHash::new(hashed_password).unwrap();
    let is_valid = argon2
        .verify_password(peppered.as_bytes(), &password_hash)
        .is_ok();
    Ok(is_valid)
}

pub async fn password_test_async(password: &str, hashed_password: &str) -> Result<bool> {
    let password = password.to_string();
    let hashed_password = hashed_password.to_string();
    let result = task::spawn_blocking(move || {
        password_test(&password, &hashed_password)
    }).await?;

    result
}

pub fn lazy_password_hash(password: &str) -> Result<String> {
    let hash_result = murmur3_x86_128(&mut Cursor::new(password), 0).expect("hashing works");
    Ok(hash_result.to_string())
}

pub async fn lazy_password_hash_async(password: &str) -> Result<String> {
    let password = password.to_string();
    let result = task::spawn_blocking(move || {
        lazy_password_hash(&password)
    }).await?;

    result
}

pub fn lazy_password_test(password: &str, hashed_password: &str) -> Result<bool> {
    let hash_result = murmur3_x86_128(&mut Cursor::new(password), 0).expect("hashing works");
    Ok(hash_result.to_string() == hashed_password)
}

pub async fn lazy_password_test_async(password: &str, hashed_password: &str) -> Result<bool> {
    let password = password.to_string();
    let hashed_password = hashed_password.to_string();
    let result = task::spawn_blocking(move || {
        lazy_password_test(&password, &hashed_password)
    }).await?;

    result
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct InviteCode(Uuid);
impl InviteCode {
    pub fn new() -> Self {
        InviteCode(Uuid::new_v4())
    }
    pub fn from_uuid(invite_code: Uuid) -> Self {
        InviteCode(invite_code)
    }
    pub fn from_string(invite_code: &str) -> Result<Self> {
        Ok(InviteCode(Uuid::parse_str(invite_code)?))
    }
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
    pub fn to_uuid(&self) -> Uuid {
        self.0
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct UserId(Uuid);
impl UserId {
    pub fn new() -> Self {
        UserId(Uuid::new_v4())
    }
    pub fn from_uuid(user_id: Uuid) -> Self {
        UserId(user_id)
    }
    pub fn from_string(user_id: &str) -> Result<Self> {
        Ok(UserId(Uuid::parse_str(user_id)?))
    }
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
    pub fn to_uuid(&self) -> Uuid {
        self.0
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct SessionToken(Uuid);
impl SessionToken {
    pub fn new() -> Self {
        SessionToken(Uuid::new_v4())
    }
    pub fn from_uuid(session_token: Uuid) -> Self {
        SessionToken(session_token)
    }
    pub fn from_string(session_token: &str) -> Result<Self> {
        Ok(SessionToken(Uuid::parse_str(session_token)?))
    }
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
    pub fn to_uuid(&self) -> Uuid {
        self.0
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserCreate<'r>{
    pub user_id: UserId,
    pub parent_id: UserId,
    pub display_name: &'r str,
    pub email: &'r str,
    pub password: &'r str,
    pub is_verified: bool,
    pub is_admin: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserSession {
    pub user_id: UserId,
    pub display_name: String,
    pub thumbnail_url: String,
    pub is_verified: bool,
    pub is_admin: bool,
    pub is_known_ip: bool,
    pub ip: IpAddr,
    pub tags: Option<Vec<String>>,
}

impl UserSession {
    pub fn to_verified_user_session(&self) -> VerifiedUserSession {
        VerifiedUserSession {
            user_id: self.user_id,
            display_name: self.display_name.clone(),
            thumbnail_url: self.thumbnail_url.clone(),
            is_admin: self.is_admin,
            tags: self.tags.clone(),
        }
    }
    pub fn to_admin_user_session(&self) -> AdminUserSession {
        AdminUserSession {
            user_id: self.user_id,
            display_name: self.display_name.clone(),
            thumbnail_url: self.thumbnail_url.clone(),
            tags: self.tags.clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerifiedUserSession {
    pub user_id: UserId,
    pub display_name: String,
    pub thumbnail_url: String,
    pub is_admin: bool,
    pub tags: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdminUserSession {
    pub user_id: UserId,
    pub display_name: String,
    pub thumbnail_url: String,
    pub tags: Option<Vec<String>>,
}

#[derive(FromRow)]
pub struct UserDatabaseRaw {
    pub id: Uuid,
    pub display_name: String,
    pub parent_id: Option<Uuid>,
    pub hashed_password: String,
    pub email: String,
    pub thumbnail_url: String,
    pub is_verified: bool,
    pub is_admin: bool,
    pub tags: Option<Vec<String>>,
    pub created_at: Duration,
    pub updated_at: Duration,
}

impl Services {
    pub async fn get_invite_code_source(
        &self,
        invite_code: &InviteCode,
    ) -> Result<UserId> {
        if invite_code.to_uuid() == ROOT_USER_ID.to_uuid(){
            return Err(anyhow!("Invalid invite code"));
        }
        Ok(ROOT_USER_ID)
    }

    pub async fn exhaust_invite_code(
        &self,
        _invite_code: &InviteCode,
    ) -> Result<()> {
        // the invite code can only be used once
        // so we'll just delete it
        Ok(())
    }

    pub async fn generate_invite_code(
        &self,
    ) -> Result<InviteCode> {
        // for testing, generate a new invite code from the root user
        Ok(InviteCode::new())
    }

    pub async fn get_my_invites(
        &self,
        _user_id: &UserId,
    ) -> Result<Vec<InviteCode>> {
        // for testing, generate a new invite code from the root user
        Ok(vec![])
    }

    pub async fn get_user_exists(
        &self,
        user_id: &UserId,
    ) -> Result<bool> {
        let result = self.scylla
            .session
            .execute(
                &self
                    .scylla
                    .prepared_queries
                    .get("get_user")
                    .expect("Query missing!"),
                (user_id.0,),
            )
            .await?;

        if let Some(rows) = result.rows {
            if rows.len() > 0 {
                return Ok(true);
            }
            else{
                return Ok(false);
            }
        }
        else{
            return Ok(false);
        }
    }

    pub async fn get_user(
        &self,
        user_id: &UserId,
    ) -> Result<Option<UserDatabaseRaw>> {
        Ok(self.scylla
            .session
            .execute(
                &self
                    .scylla
                    .prepared_queries
                    .get("get_user")
                    .expect("Query missing!"),
                (user_id.to_uuid(),),
            )
            .await?
            .maybe_first_row_typed::<UserDatabaseRaw>()?)
    }

    pub async fn get_user_email(
        &self,
        email: &str,
    ) -> Result<Option<UserDatabaseRaw>> {
        let result = self.scylla
            .session
            .execute(
                &self
                    .scylla
                    .prepared_queries
                    .get("get_email_user")
                    .expect("Query missing!"),
                (email,),
            )
            .await?;

        if let Some(rows) = result.rows {
            if rows.len() > 0 {
                let row = rows.get(0).unwrap();
                let user_id: Uuid = row.columns[0].as_ref().unwrap().as_uuid().unwrap();
                let user_id = UserId(user_id);
                return self.get_user(&user_id).await;
            }
            else{
                return Ok(None);
            }
        }
        else{
            return Ok(None);
        }
    }

    pub async fn create_root_user(&self) -> Result<()>{
        // don't create a root user if one already exists
        if self.get_user_exists(&ROOT_USER_ID).await? {
            return Ok(());
        }

        let user_id = ROOT_USER_ID.to_uuid();
        let display_name = "root";
        let email = "root@gooble.email";
        let parent_id = "";
        let root_auth_password = env::var("GROOVELET_ROOT_AUTH_PASSWORD").unwrap_or_else(|_| "root".to_string());

        let hashed_password: String;
        if self.is_production{
            hashed_password = password_hash_async(&root_auth_password).await?;
        }
        else{
            hashed_password = lazy_password_hash_async(&root_auth_password).await?;
        }

        self.scylla
            .session
            .execute(
                &self
                    .scylla
                    .prepared_queries
                    .get("create_user")
                    .expect("Query missing!"),
                //.prepare("INSERT INTO ks.user (id, display_name, parent_id, hashed_password, email, thumbnail_url, is_verified, is_admin, created_at, updated_at) VALUES (?, ?, ?, ?, ?, false, ?, ?);")
                (user_id, display_name, parent_id, hashed_password, email, DEFAULT_THUMBNAIL_URL, true, true, Utc::now().timestamp_millis(), Utc::now().timestamp_millis()),
            )
            .await?;

        // email -> user
        self.scylla
            .session
            .execute(
                &self
                    .scylla
                    .prepared_queries
                    .get("set_email_user")
                    .expect("query missing!"),
                (email, user_id,),
            )
            .await?;

        Ok(())
    }

    pub async fn create_user(
        &self,
        user_create: UserCreate<'_>,
        ip: IpAddr,
    ) -> Result<SessionToken> {
        if self.get_user_exists(&user_create.user_id).await? {
            return Err(anyhow!("User somehow already exists! Wow, UUIDs are not as unique as I thought!"));
        }
        if !self.get_user_exists(&user_create.parent_id).await? {
            return Err(anyhow!("Parent user does not exist!"));
        }
        let email_user = self.get_user_email(&user_create.email).await?;
        if let Some(email_user) = email_user {
            if email_user.is_verified{
                return Err(anyhow!("Email already exists!"));
            }
            else{
                // TODO: delete the unverified user
                // and just create a new one, now
                // suck it, chump
                self.delete_user(&UserId::from_uuid(email_user.id)).await?;
            }
        }

        let hashed_password: String;
        if self.is_production{
            hashed_password = password_hash_async(&user_create.password).await?;
        }
        else{
            hashed_password = lazy_password_hash_async(&user_create.password).await?;
        }

        let user_id = user_create.user_id.to_uuid();

        // the core user record!
        self.scylla
            .session
            .execute(
                &self
                    .scylla
                    .prepared_queries
                    .get("create_user")
                    .expect("Query missing!"),
                (
                    user_id,
                    user_create.display_name,
                    user_create.parent_id.0,
                    hashed_password,
                    user_create.email,
                    DEFAULT_THUMBNAIL_URL,
                    user_create.is_verified,
                    user_create.is_admin,
                    Utc::now().timestamp_millis(),
                    Utc::now().timestamp_millis()
                ),
            )
            .await?;

        // email -> user
        self.scylla
            .session
            .execute(
                &self
                    .scylla
                    .prepared_queries
                    .get("set_email_user")
                    .expect("query missing!"),
                (user_create.email, user_create.user_id.0,),
            )
            .await?;

        // user -> ip
        self.scylla
            .session
            .execute(
                &self
                    .scylla
                    .prepared_queries
                    .get("set_user_ip")
                    .expect("Query missing!"),
                (user_id, ip, ),
            )
            .await?;

        self.send_verification_email( &user_create.user_id.0, &user_create.email ).await?;

        let user_session = UserSession{
            user_id: user_create.user_id,
            display_name: user_create.display_name.to_string(),
            thumbnail_url: DEFAULT_THUMBNAIL_URL.to_string(),
            is_verified: user_create.is_verified,
            is_admin: user_create.is_admin,
            is_known_ip: true,
            ip: ip,
            tags: None,
        };

        let session_token = self.create_session_token(&user_session).await?;

        Ok(session_token)
    }

    pub async fn is_this_a_known_ip_for_this_user(
        &self,
        user_id: &UserId,
        ip: &IpAddr,
    ) -> Result<bool> {
        let result = self.scylla
            .session
            .execute(
                &self
                    .scylla
                    .prepared_queries
                    .get("get_user_ip")
                    .expect("Query missing!"),
                (user_id.to_uuid(), ip,),
            )
            .await?;

        if let Some(rows) = result.rows {
            if rows.len() > 0 {
                return Ok(true);
            }
            else{
                return Ok(false);
            }
        }
        else{
            return Ok(false);
        }
    }

    pub async fn login(&self, email: &str, password: &str, ip: IpAddr) -> Result<SessionToken> {
        let email_user = self.get_user_email(&email).await?;
        if let Some(email_user) = email_user {
            let password_success:bool;
            if self.is_production {
                password_success = password_test_async(&password, &email_user.hashed_password).await?;
            }
            else{
                password_success = lazy_password_test_async(&password, &email_user.hashed_password).await?;
            }

            let known_ip = self.is_this_a_known_ip_for_this_user(&UserId::from_uuid(email_user.id), &ip).await?;

            if !known_ip {
                self.send_ip_verification_email(&email_user.id, &email).await?;
            }

            if password_success {
                let user_id: UserId = UserId::from_uuid(email_user.id);
                let user_session: UserSession = UserSession{
                    user_id: user_id,
                    display_name: email_user.display_name,
                    thumbnail_url: email_user.thumbnail_url,
                    is_verified: email_user.is_verified,
                    is_admin: email_user.is_admin,
                    is_known_ip: known_ip,
                    ip: ip,
                    tags: email_user.tags,
                };

                let session_token = self.create_session_token(&user_session).await?;
                return Ok(session_token);
            }
        }
        Err(anyhow!("Invalid email or password!"))
    }

    pub async fn delete_user(
        &self,
        user_id: &UserId,
    ) -> Result<()> {
        self.scylla
            .session
            .execute(
                &self
                    .scylla
                    .prepared_queries
                    .get("delete_user")
                    .expect("Query missing!"),
                (user_id.to_uuid(),),
            )
            .await?;

        Ok(())
    }

/*
          _______  _______ _________ _______ _________ _______  _______ __________________ _______  _
|\     /|(  ____ \(  ____ )\__   __/(  ____ \\__   __/(  ____ \(  ___  )\__   __/\__   __/(  ___  )( (    /|
| )   ( || (    \/| (    )|   ) (   | (    \/   ) (   | (    \/| (   ) |   ) (      ) (   | (   ) ||  \  ( |
| |   | || (__    | (____)|   | |   | (__       | |   | |      | (___) |   | |      | |   | |   | ||   \ | |
( (   ) )|  __)   |     __)   | |   |  __)      | |   | |      |  ___  |   | |      | |   | |   | || (\ \) |
 \ \_/ / | (      | (\ (      | |   | (         | |   | |      | (   ) |   | |      | |   | |   | || | \   |
  \   /  | (____/\| ) \ \_____) (___| )      ___) (___| (____/\| )   ( |   | |   ___) (___| (___) || )  \  |
   \_/   (_______/|/   \__/\_______/|/       \_______/(_______/|/     \|   )_(   \_______/(_______)|/    )_)

*/

    pub async fn test_get_last_email(&self, email_address: &str) -> Result<String> {
        let mut redis_connection = self.application_redis.get_async_connection().await?;
        let last_email_sent_key = format!("last_email_sent:${}", email_address);
        let last_email_sent: String = redis_connection.get(&last_email_sent_key).await?;
        Ok(last_email_sent)
    }

    pub async fn send_verification_email(
        &self,
        user_id: &Uuid,
        email_address: &str,
    ) -> Result<()> {
        let mut redis_connection = self.application_redis.get_async_connection().await?;
        let email_verification_token = Uuid::new_v4().to_string();
        let key = format!("email_verification_token:${}", email_verification_token);

        redis_connection.set_ex(&key, user_id.to_string(), EMAIL_VERIFICATION_TIMEOUT_SECONDS).await?;

        let public_address = self.config_get_public_address();

        let email_verification_link = format!("{}/auth/verify_email?token={}", public_address, email_verification_token);

        self.email.send_verification_email(&EmailAddress::new(email_address.to_string())?, &email_verification_link).await?;

        if ! self.is_production {
            let last_email_sent_key = format!("last_email_sent:${}", email_address);
            redis_connection.set(&last_email_sent_key, email_verification_link).await?;
        }

        Ok(())
    }

    pub async fn send_ip_verification_email(
        &self,
        user_id: &Uuid,
        email_address: &str,
    ) -> Result<()> {
        let mut redis_connection = self.application_redis.get_async_connection().await?;
        let email_verification_token = Uuid::new_v4().to_string();
        let key = format!("ip_verification_token:${}", email_verification_token);

        redis_connection.set_ex(&key, user_id.to_string(), EMAIL_VERIFICATION_TIMEOUT_SECONDS).await?;

        let public_address = self.config_get_public_address();

        let ip_verification_link = format!("{}/auth/verify_ip?token={}", public_address, email_verification_token);

        self.email.send_ip_verification_email(&EmailAddress::new(email_address.to_string())?, &ip_verification_link).await?;

        if ! self.is_production {
            let last_email_sent_key = format!("last_email_sent:${}", email_address);
            redis_connection.set(&last_email_sent_key, ip_verification_link).await?;
        }

        Ok(())
    }

    pub async fn verify_email(
        &self,
        email_verification_token: &Uuid,
    ) -> Result<UserId> {
        let mut redis_connection = self.application_redis.get_async_connection().await?;
        let verification_token_key = format!("email_verification_token:${}", email_verification_token.to_string());
        let user_id: String = redis_connection.get(&verification_token_key).await?;
        let user_id = Uuid::parse_str(&user_id)?;
        let user_id = UserId(user_id);

        if ! self.get_user_exists(&user_id).await? {
            return Err(anyhow!("User does not exist!"));
        }

        self.scylla
            .session
            .execute(
                &self
                    .scylla
                    .prepared_queries
                    .get("verify_user_email")
                    .expect("Query missing!"),
                (user_id.to_uuid(),),
            )
            .await?;

        self.verify_all_sessions(&user_id).await?;

        redis_connection.unlink(&verification_token_key).await?;

        Ok(user_id)
    }

    pub async fn remember_ip(
        &self,
        user_id: &UserId,
        ip: &IpAddr,
    ) -> Result<()> {
        self.scylla
            .session
            .execute(
                &self
                    .scylla
                    .prepared_queries
                    .get("set_user_ip")
                    .expect("Query missing!"),
                (user_id.to_uuid(), ip, ),
            )
            .await?;

        Ok(())
    }

    pub async fn verify_ip(
        &self,
        email_verification_token: &Uuid,
        ip: &IpAddr,
    ) -> Result<()> {
        let mut redis_connection = self.application_redis.get_async_connection().await?;
        let verification_token_key = format!("ip_verification_token:${}", email_verification_token.to_string());
        let user_id: String = redis_connection.get(&verification_token_key).await?;
        let user_id = Uuid::parse_str(&user_id)?;
        let user_id = UserId(user_id);

        if ! self.get_user_exists(&user_id).await? {
            return Err(anyhow!("User does not exist!"));
        }

        self.remember_ip(&user_id, &ip).await?;

        self.verify_ip_all_sessions(&user_id, &ip).await?;

        redis_connection.unlink(&verification_token_key).await?;

        Ok(())
    }

    pub async fn forget_ip(
        &self,
        user_id: &UserId,
        ip: &IpAddr,
    ) -> Result<()> {

        self.scylla
            .session
            .execute(
                &self
                    .scylla
                    .prepared_queries
                    .get("delete_user_ip")
                    .expect("Query missing!"),
                (user_id.to_uuid(), ip,),
            ).await?;

        Ok(())
    }

/*
                                 _                    _
 ___ ___ ___ ___ _ _ _ ___ ___ _| |   ___ ___ ___ ___| |_
| . | .'|_ -|_ -| | | | . |  _| . |  |  _| -_|_ -| -_|  _|
|  _|__,|___|___|_____|___|_| |___|  |_| |___|___|___|_|
|_|
*/

    pub async fn send_password_reset_email(
        &self,
        email_address: &str,
    ) -> Result<()> {

        let user_maybe = self.get_user_email(&email_address).await?;
        match user_maybe {
            None => {
                Err(anyhow!("User does not exist!"))
            },
            Some(user) => {
                let user_id = user.id;

                let mut redis_connection = self.application_redis.get_async_connection().await?;
                let password_reset_token = Uuid::new_v4().to_string();
                let key = format!("password_reset_token:${}", password_reset_token);

                redis_connection.set_ex(&key, user_id.to_string(), PASSWORD_RESET_TIMEOUT_SECONDS).await?;

                let public_address = self.config_get_public_address();

                let password_reset_link = format!("{}/auth/password_reset/stage_2?token={}", public_address, password_reset_token);

                self.email.send_password_reset_email(&EmailAddress::new(email_address.to_string())?, &password_reset_link).await?;

                if ! self.is_production {
                    let last_email_sent_key = format!("last_email_sent:${}", email_address);
                    redis_connection.set(&last_email_sent_key, password_reset_link).await?;
                }

                Ok(())
            }
        }
    }

    pub async fn password_reset(&self, password_token: &Uuid, password: &str, ip: &IpAddr) -> Result<SessionToken> {
        // 1. verify the token and find the associated user id
        let mut redis_connection = self.application_redis.get_async_connection().await?;
        let verification_token_key = format!("password_reset_token:${}", password_token.to_string());
        let user_id: String = redis_connection.get(&verification_token_key).await?;
        let user_id = Uuid::parse_str(&user_id)?;
        let user_id = UserId(user_id);

        // 2. hash the password and save it against the associated user id
        let hashed_password: String;
        if self.is_production{
            hashed_password = password_hash_async(&password).await?;
        }
        else{
            hashed_password = lazy_password_hash_async(&password).await?;
        }

        self.scylla
            .session
            .execute(
                &self
                    .scylla
                    .prepared_queries
                    .get("change_user_password")
                    .expect("Query missing!"),
                (hashed_password, user_id.to_uuid(),),
            ).await?;

        // 3. while we're here, save that IP as a known IP for this user
        self.remember_ip(&user_id, &ip).await?;

        // 4. get that user, create a session token, and return it
        let user = self.get_user(&user_id).await?.unwrap();
        let user_session = UserSession{
            user_id: UserId::from_uuid(user.id),
            display_name: user.display_name,
            thumbnail_url: user.thumbnail_url,
            is_verified: user.is_verified,
            is_admin: user.is_admin,
            is_known_ip: true,
            ip: *ip,
            tags: user.tags,
        };

        let session_token = self.create_session_token(&user_session).await?;

        Ok(session_token)
    }


/*
 ______     ______     ______   ______        __         __     __    __     __     ______   ______
/\  == \   /\  __ \   /\__  _\ /\  ___\      /\ \       /\ \   /\ "-./  \   /\ \   /\__  _\ /\  ___\
\ \  __<   \ \  __ \  \/_/\ \/ \ \  __\      \ \ \____  \ \ \  \ \ \-./\ \  \ \ \  \/_/\ \/ \ \___  \
 \ \_\ \_\  \ \_\ \_\    \ \_\  \ \_____\     \ \_____\  \ \_\  \ \_\ \ \_\  \ \_\    \ \_\  \/\_____\
  \/_/ /_/   \/_/\/_/     \/_/   \/_____/      \/_____/   \/_/   \/_/  \/_/   \/_/     \/_/   \/_____/

*/


    pub async fn rate_limit(&self, key: &String, requests_per_hour: usize) -> Result<()> {
        /*
            Whatever the key is, it's not allowed to call this funciton more than requests_per_hour times per hour,
            if it does, it'll throw a rate limit error.
            It also can't call this function more than once every 5 seconds.
        */
        let mut redis_connection = self.application_redis.get_async_connection().await?;

        // everything has a 5-second rate limit by default
        let rate_limit_key = format!("rate_limit:${}", key);
        let rate_limit_exists: bool = redis_connection.exists(&rate_limit_key).await?;
        if rate_limit_exists {
            return Err(anyhow!("Rate limit exceeded!"));
        }
        redis_connection.set_ex(&rate_limit_key, "NO", 5).await?;

        // everything also gets no more than requests_per_hour requests per hour
        let rate_limit_key = format!("rate_limit_hour:${}", key);
        let rate_limit_exists: bool = redis_connection.exists(&rate_limit_key).await?;
        if !rate_limit_exists {
            redis_connection.set_ex(&rate_limit_key, 0, 3600).await?;
        }
        else{
            let rate_limit_count: usize = redis_connection.incr(&rate_limit_key, 1).await?;
            if rate_limit_count > requests_per_hour {
                return Err(anyhow!("Rate limit exceeded!"));
            }
        }

        Ok(())
    }

    pub async fn rate_limits(&self, keys: &Vec<String>, requests_per_hour: usize) -> Result<()> {
        /*
            Apply multiple rate limits at once.
        */
        for key in keys {
            self.rate_limit(key, requests_per_hour).await?;
        }
        Ok(())
    }

/*


                                                                         iiii
                                                                        i::::i
                                                                         iiii

    ssssssssss       eeeeeeeeeeee        ssssssssss       ssssssssss   iiiiiii    ooooooooooo   nnnn  nnnnnnnn
  ss::::::::::s    ee::::::::::::ee    ss::::::::::s    ss::::::::::s  i:::::i  oo:::::::::::oo n:::nn::::::::nn
ss:::::::::::::s  e::::::eeeee:::::eess:::::::::::::s ss:::::::::::::s  i::::i o:::::::::::::::on::::::::::::::nn
s::::::ssss:::::se::::::e     e:::::es::::::ssss:::::ss::::::ssss:::::s i::::i o:::::ooooo:::::onn:::::::::::::::n
 s:::::s  ssssss e:::::::eeeee::::::e s:::::s  ssssss  s:::::s  ssssss  i::::i o::::o     o::::o  n:::::nnnn:::::n
   s::::::s      e:::::::::::::::::e    s::::::s         s::::::s       i::::i o::::o     o::::o  n::::n    n::::n
      s::::::s   e::::::eeeeeeeeeee        s::::::s         s::::::s    i::::i o::::o     o::::o  n::::n    n::::n
ssssss   s:::::s e:::::::e           ssssss   s:::::s ssssss   s:::::s  i::::i o::::o     o::::o  n::::n    n::::n
s:::::ssss::::::se::::::::e          s:::::ssss::::::ss:::::ssss::::::si::::::io:::::ooooo:::::o  n::::n    n::::n
s::::::::::::::s  e::::::::eeeeeeee  s::::::::::::::s s::::::::::::::s i::::::io:::::::::::::::o  n::::n    n::::n
 s:::::::::::ss    ee:::::::::::::e   s:::::::::::ss   s:::::::::::ss  i::::::i oo:::::::::::oo   n::::n    n::::n
  sssssssssss        eeeeeeeeeeeeee    sssssssssss      sssssssssss    iiiiiiii   ooooooooooo     nnnnnn    nnnnnn


*/

    pub async fn create_session_token(&self, user_session: &UserSession) -> Result<SessionToken>{
        /*
            given a user session, create a session token and save it in redis
         */
        let mut redis_connection = self.application_redis.get_async_connection().await?;
        let session_token = Uuid::new_v4();
        let session_json = serde_json::to_string(user_session)?;
        let key = format!("session_token:{}", session_token.to_string());
        redis_connection.set_ex(&key, session_json, USER_SESSION_TIMEOUT_SECONDS).await?;

        let user_sessions_key = format!("user_sessions:{}", user_session.user_id.to_string());
        redis_connection.zadd(&user_sessions_key, session_token.to_string(), Utc::now().timestamp_millis()).await?;
        redis_connection.expire(&user_sessions_key, USER_SESSION_TIMEOUT_SECONDS*2).await?;

        let user_sessions_count: usize = redis_connection.zcard(&user_sessions_key).await?;
        //println!("user_sessions_count: {}", user_sessions_count);
        // if the user has more than MAX_SESSIONS sessions, delete the oldest one
        if user_sessions_count > USER_MAX_SESSION_COUNT {
            self.cull_old_sessions(&user_session.user_id).await?;
        }

        Ok(SessionToken(session_token))
    }

    pub async fn cull_old_sessions(&self, user_id: &UserId) -> Result<()>{
        // the user has more than USER_MAX_SESSION_COUNT sessions, delete all but the USER_MAX_SESSION_COUNT most recent
        // it's also fine to cull any that have obviously expired (> USER_SESSION_TIMEOUT_SECONDS old)
        let timestamp_cutoff: i64 = Utc::now().timestamp_millis() - (USER_SESSION_TIMEOUT_SECONDS as i64 * 1000);

        let mut counter: usize = 0;
        for (session_token, timestamp) in self.get_all_sessions(&user_id).await? {
            if timestamp < timestamp_cutoff || counter > USER_MAX_SESSION_COUNT {
                self.delete_session(&session_token, &user_id).await?;
            }
            counter += 1;
        }

        Ok(())
    }

    /*
    pub async fn logout(&self, session_token: &SessionToken) -> Result<()>{
        let user_session = self.get_user_from_session_token(&session_token).await?;
        let user_id = user_session.user_id;
        self.delete_session(&session_token, &user_id).await?;
        Ok(())
    }
    */

    pub async fn delete_session(&self, session_token: &SessionToken, user_id: &UserId) -> Result<()>{
        let mut redis_connection = self.application_redis.get_async_connection().await?;
        redis_connection.unlink(&format!("session_token:{}", session_token.to_string())).await?;
        redis_connection.zrem(&format!("user_sessions:{}", user_id.to_string()), session_token.to_string()).await?;

        Ok(())
    }

    pub async fn delete_all_sessions(&self, user_id: &UserId) -> Result<()>{
        for (session_token, _timestamp) in self.get_all_sessions(&user_id).await? {
            self.delete_session(&session_token, &user_id).await?;
        }
        Ok(())
    }

    pub async fn verify_session(&self, session_token: &SessionToken) -> Result<()>{
        let mut redis_connection = self.application_redis.get_async_connection().await?;
        let key = format!("session_token:{}", session_token.to_string());
        let session_json: String = redis_connection.get(&key).await?;

        //println!("verifying session_json: {}", session_json);

        let mut user_session: UserSession = serde_json::from_str(&session_json)?;

        user_session.is_verified = true;

        let session_json = serde_json::to_string(&user_session)?;

        redis_connection.set_ex(&key, session_json, USER_SESSION_TIMEOUT_SECONDS).await?;

        Ok(())
    }

    pub async fn verify_session_ip(&self, session_token: &SessionToken, ip: &IpAddr) -> Result<()>{
        let mut redis_connection = self.application_redis.get_async_connection().await?;
        let key = format!("session_token:{}", session_token.to_string());
        let session_json: String = redis_connection.get(&key).await?;

        let mut user_session: UserSession = serde_json::from_str(&session_json)?;

        if user_session.ip.to_string() == ip.to_string(){
            user_session.is_known_ip = true;

            let session_json = serde_json::to_string(&user_session)?;

            redis_connection.set_ex(&key, session_json, USER_SESSION_TIMEOUT_SECONDS).await?;
        }

        Ok(())
    }

    pub async fn get_all_sessions(&self, user_id: &UserId) -> Result<Vec<(SessionToken, i64)>> {
        let mut redis_connection = self.application_redis.get_async_connection().await?;
        let user_sessions: Vec<(String, i64)> = redis_connection.zrangebyscore_withscores(&format!("user_sessions:{}", user_id.to_string()), "-inf", "+inf").await?;

        let new_user_sessions: Vec<(SessionToken, i64)> = user_sessions.iter().map(|(session_token, timestamp)| (SessionToken::from_string(&session_token).unwrap(), *timestamp)).collect();

        Ok(new_user_sessions)
    }

    pub async fn verify_all_sessions(&self, user_id: &UserId) -> Result<()>{
        /*
            after the user has verified that their email address is valid, we should verify all of their sessions
         */
        for (session_token, _timestamp) in self.get_all_sessions(&user_id).await? {
            self.verify_session(&session_token).await?;
        }
        Ok(())
    }

    pub async fn verify_ip_all_sessions(&self, user_id: &UserId, ip: &IpAddr) -> Result<()>{
        /*
            after the user has verified that their email address is valid, we should verify all of their sessions
         */
        for (session_token, _timestamp) in self.get_all_sessions(&user_id).await? {
            self.verify_session_ip(&session_token, &ip).await?;
        }
        Ok(())
    }

    pub async fn refresh_session_token(&self, session_token: &SessionToken, user_id: &UserId) -> Result<()>{
        let mut redis_connection = self.application_redis.get_async_connection().await?;

        redis_connection.expire(&format!("session_token:{}", session_token.to_string()), USER_SESSION_TIMEOUT_SECONDS).await?;

        let user_sessions_key = format!("user_sessions:{}", user_id.to_string());
        redis_connection.zadd(&user_sessions_key, session_token.to_string(), Utc::now().timestamp_millis()).await?;

        Ok(())
    }

    pub async fn get_user_from_session_token(&self, session_token: &SessionToken) -> Result<UserSession>{
        let mut redis_connection = self.application_redis.get_async_connection().await?;

        let session_json: String = redis_connection.get(&format!("session_token:{}", session_token.to_string())).await?;

        //println!("getting session_json: {}", session_json);

        let user_session: UserSession = serde_json::from_str(&session_json)?;

        // note: it may be needlessly expensive to do this every single time, presumably, users are doing this on the reg
        self.refresh_session_token(session_token, &user_session.user_id).await?;

        return Ok(user_session);
    }

}