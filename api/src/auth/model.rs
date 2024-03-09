use std::collections::HashSet;
use std::env;
use std::net::IpAddr;

use serde::{Deserialize, Serialize};

use anyhow::Result;
use anyhow::anyhow;

use rocket::serde::uuid::Uuid;
//use scylla::frame::value::Timestamp;


use crate::email::EmailAddress;
use crate::Services;
use crate::auth::hashes;
use crate::services::user_invite_service::UserInviteRaw;

const ROOT_USER_ID: UserId = UserId(Uuid::from_u128(0));
const DEFAULT_THUMBNAIL_URL: &str = "/static/chismas.png";


#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq)]
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

/*
#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct Invite{
    pub invite_code: InviteCode,
    pub is_used: bool,
}
*/

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq)]
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
    pub tags: HashSet<String>,
}

impl crate::services::auth_token_service::HasUserId for UserSession{
    fn user_id(&self) -> Uuid{
        self.user_id.to_uuid()
    }
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
    pub tags: HashSet<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdminUserSession {
    pub user_id: UserId,
    pub display_name: String,
    pub thumbnail_url: String,
    pub tags: HashSet<String>,
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

    pub async fn create_invite_code(
        &self,
        user_id: &UserId,
    ) -> Result<()> {
        // first we need to check if the user has any available invites
        let user_maybe = self.user_service.get_user(&user_id).await?;
        let invite_count = self.user_invite_service.count_invites(&user_id).await?;
        match user_maybe {
            None => {
                Err(anyhow!("User does not exist!"))
            },
            Some(user) => {
                let available_invites = user.available_user_invites();
                if available_invites < invite_count {
                    return Err(anyhow!("No available invites!"));
                }
                else{
                    self.user_invite_service.create_invite(&user_id).await?;
                    Ok(())
                }
            }
        }
    }

    pub async fn delete_invite_code(
        &self,
        user_id: &UserId,
        invite_code: &InviteCode,
    ) -> Result<()> {
        let invite = self.user_invite_service.get_invite(&invite_code).await?;

        match invite {
            None => {
                return Err(anyhow!("Invite code does not exist!"));
            },
            Some(invite) => {
                if invite.user_id != *user_id {
                    return Err(anyhow!("You can't delete that invite code! It's not yours!"));
                }
                self.user_invite_service.delete_invite(&invite_code).await?;
                Ok(())
            }
        }
    }

    pub async fn get_my_invites(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<UserInviteRaw>> {
        // for testing, generate a new invite code from the root user
        self.user_invite_service.get_invites(&user_id).await
    }

    pub async fn get_number_available_invites(
        &self,
        user_id: &UserId) -> Result<i32> {
        let user_maybe = self.user_service.get_user(&user_id).await?;
        match user_maybe {
            None => {
                Err(anyhow!("User does not exist!"))
            },
            Some(user) => {
                Ok(user.available_user_invites())
            }
        }
    }

    pub async fn create_root_user(&self) -> Result<()>{
        // don't create a root user if one already exists
        if self.user_service.user_exists(&ROOT_USER_ID)?{
            return Ok(());
        }

        let display_name = "root".to_string();
        let email = env::var("GROOVELET_ROOT_EMAIL").unwrap_or_else(|_| "root@gooble.email".to_string());
        let root_auth_password = env::var("GROOVELET_ROOT_AUTH_PASSWORD").unwrap_or_else(|_| "root".to_string());

        let hashed_password: String = hashes::password_hash_async(&root_auth_password).await?;

        let user_to_create = crate::services::user_service::UserDatabaseCreate{
            id: ROOT_USER_ID,
            parent_id: None,
            thumbnail_url: DEFAULT_THUMBNAIL_URL.to_string(),
            tags: HashSet::new(),
            display_name,
            email,
            hashed_password,
            is_verified: true,
            is_admin: true,
        };

        self.user_service.create_user(user_to_create).await?;

        Ok(())
    }

    pub async fn create_user(
        &self,
        user_create: UserCreate<'_>,
        ip: IpAddr,
    ) -> Result<SessionToken> {
        if self.user_service.user_exists(&user_create.user_id)? {
            return Err(anyhow!("User somehow already exists! Wow, UUIDs are not as unique as I thought!"));
        }
        if !self.user_service.user_exists(&user_create.parent_id)? {
            return Err(anyhow!("Parent user does not exist!"));
        }
        let existing_user_with_same_email = self.user_service.get_user_by_email(user_create.email).await?;
        if let Some(existing_user_with_same_email) = existing_user_with_same_email {
            if existing_user_with_same_email.is_verified{
                return Err(anyhow!("Email already exists!"));
            }
            else{
                // delete the unverified user
                // and just create a new one, now
                // suck it, chump
                self.user_service.delete_user(&existing_user_with_same_email.id).await?;
            }
        }

        let hashed_password: String = hashes::password_hash_async(&user_create.password).await?;

        let user_to_create = crate::services::user_service::UserDatabaseCreate{
            id: user_create.user_id,
            parent_id: Some(user_create.parent_id),
            thumbnail_url: DEFAULT_THUMBNAIL_URL.to_string(),
            tags: HashSet::new(),
            display_name: user_create.display_name.to_string(),
            email: user_create.email.to_string(),
            hashed_password: hashed_password.to_string(),
            is_verified: user_create.is_verified,
            is_admin: user_create.is_admin,
        };

        self.user_service.create_user(user_to_create).await?;

        self.send_verification_email( &user_create.user_id, &user_create.email ).await?;

        let user_session = UserSession{
            user_id: user_create.user_id,
            display_name: user_create.display_name.to_string(),
            thumbnail_url: DEFAULT_THUMBNAIL_URL.to_string(),
            is_verified: user_create.is_verified,
            is_admin: user_create.is_admin,
            is_known_ip: true,
            ip: ip,
            tags: HashSet::new(),
        };

        let session_token = self.create_session_token(&user_session).await?;

        Ok(session_token)
    }

    pub async fn login(&self, email: &str, password: &str, ip: IpAddr) -> Result<SessionToken> {
        let email_user = self.user_service.get_user_by_email(email).await?;
        if let Some(email_user) = email_user {
            let password_success:bool = hashes::password_test_async(&password, &email_user.hashed_password).await?;

            let known_ip = self.user_service.user_has_used_ip(&email_user.id, &ip)?;

            if !known_ip {
                self.send_ip_verification_email(&email_user.id, &email).await?;
            }
            else{
                println!("User has logged in with ip {} before", ip.to_string());
            }


            if password_success {
                let user_session: UserSession = UserSession{
                    user_id: email_user.id,
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

    pub async fn test_get_last_email(&self, email_address: &str) -> Option<String> {
        let last_email_sent_key = format!("last_email_sent:${}", email_address);
        self.local_cache.get(&last_email_sent_key).await
    }

    pub async fn send_verification_email(
        &self,
        user_id: &UserId,
        email_address: &str,
    ) -> Result<()> {
        let email_verification_token = self.email_token_service.create_token(user_id.clone()).await?;

        let public_address = crate::config::public_address();

        let email_verification_link = format!("{}/auth/verify_email?token={}", public_address, email_verification_token);

        self.email.send_verification_email(&EmailAddress::new(email_address.to_string())?, &email_verification_link).await?;

        // we keep track of the last email sent, so that we can test this functionality
        if ! self.is_production {
            let last_email_sent_key = format!("last_email_sent:${}", email_address);
            self.local_cache.insert(last_email_sent_key, email_verification_link).await;
        }

        Ok(())
    }

    pub async fn resend_verification_email(
        &self,
        user_id: &UserId,
    ) -> Result<()> {
        match self.user_service.get_user(&user_id).await? {
            None => {
                Err(anyhow!("User does not exist!"))
            },
            Some(user) => {
                if user.is_verified {
                    Err(anyhow!("User is already verified!"))
                }
                else{
                    self.send_verification_email(&user_id, &user.email).await?;
                    Ok(())
                }
            }
        }
    }

    pub async fn send_ip_verification_email(
        &self,
        user_id: &UserId,
        email_address: &str,
    ) -> Result<()> {
        let email_verification_token = self.ip_token_service.create_token(user_id.clone()).await?;

        let public_address = crate::config::public_address();

        let ip_verification_link = format!("{}/auth/verify_ip?token={}", public_address, email_verification_token);

        self.email.send_ip_verification_email(&EmailAddress::new(email_address.to_string())?, &ip_verification_link).await?;

        if ! self.is_production {
            let last_email_sent_key = format!("last_email_sent:${}", email_address);
            self.local_cache.insert(last_email_sent_key, ip_verification_link).await;
        }

        Ok(())
    }

    pub async fn resend_ip_verification_email(
        &self,
        user_id: &UserId,
    ) -> Result<()> {
        match self.user_service.get_user(&user_id).await? {
            None => {
                Err(anyhow!("User does not exist!"))
            },
            Some(user) => {
                self.send_ip_verification_email(&user_id, &user.email).await?;
                Ok(())
            }
        }
    }

    pub async fn verify_email(
        &self,
        email_verification_token: &Uuid,
    ) -> Result<UserId> {
        let user_id = self.email_token_service.get_token(&email_verification_token).await?;
        match user_id {
            None => {
                Err(anyhow!("Invalid token!"))
            },
            Some(user_id) => {
                self.user_service.verify_user(&user_id).await?;

                self.verify_all_sessions(&user_id).await?;

                self.email_token_service.delete_token(&email_verification_token).await?;

                Ok(user_id)
            }
        }
    }

    pub async fn remember_ip(
        &self,
        user_id: &UserId,
        ip: &IpAddr,
    ) -> Result<()> {
        self.user_service.set_user_ip(&user_id, ip.clone()).await?;

        Ok(())
    }

    pub async fn verify_ip(
        &self,
        email_verification_token: &Uuid,
        ip: &IpAddr,
    ) -> Result<()> {
        let user_id = self.ip_token_service.get_token(&email_verification_token).await?;
        match user_id {
            None => {
                Err(anyhow!("Invalid token!"))
            },
            Some(user_id) => {

                self.user_service.set_user_ip(&user_id, ip.clone()).await?;

                self.verify_ip_all_sessions(&user_id, &ip).await?;

                self.ip_token_service.delete_token(&email_verification_token).await?;

                Ok(())
            }
        }
    }

    pub async fn forget_ip(
        &self,
        user_id: &UserId,
        ip: &IpAddr,
    ) -> Result<()> {
        self.user_service.delete_ip(&user_id, &ip).await?;

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

        match self.user_service.get_user_by_email(&email_address).await? {
            None => {
                Err(anyhow!("User does not exist!"))
            },
            Some(user) => {
                let password_reset_token = self.password_token_service.create_token(user.id).await?;

                let public_address = crate::config::public_address();

                let password_reset_link = format!("{}/auth/password_reset/stage_2?token={}", public_address, password_reset_token);

                self.email.send_password_reset_email(&EmailAddress::new(email_address.to_string())?, &password_reset_link).await?;

                if ! self.is_production {
                    let last_email_sent_key = format!("last_email_sent:${}", email_address);
                    self.local_cache.insert(last_email_sent_key, password_reset_link).await;
                }

                Ok(())
            }
        }
    }

    pub async fn password_reset(&self, password_token: &Uuid, password: &str, ip: &IpAddr) -> Result<SessionToken> {
        // 1. verify the token and find the associated user id
        let user_id_maybe = self.password_token_service.get_token(&password_token).await?;

        match user_id_maybe{
            None => {
                Err(anyhow!("Invalid token!"))
            },
            Some(user_id) => {
                if ! self.user_service.user_exists(&user_id)? {
                    return Err(anyhow!("User does not exist!"));
                }

                // 2. hash the password and save it against the associated user id
                let hashed_password: String = hashes::password_hash_async(&password).await?;

                self.user_service.change_password(&user_id, &hashed_password).await?;

                // 3. while we're here, save that IP as a known IP for this user
                self.user_service.set_user_ip(&user_id, ip.clone()).await?;

                // 4. get that user, create a session token, and return it
                match self.user_service.get_user(&user_id).await?{
                    None => {
                        Err(anyhow!("User does not exist!"))
                    },
                    Some(user) => {
                        let user_session = UserSession{
                            user_id: user.id,
                            display_name: user.display_name,
                            thumbnail_url: user.thumbnail_url,
                            is_verified: user.is_verified,
                            is_admin: user.is_admin,
                            is_known_ip: true,
                            ip: ip.clone(),
                            tags: user.tags,
                        };

                        let session_token = self.create_session_token(&user_session).await?;

                        Ok(session_token)
                    }
                }
            }
        }
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

        let session_token = self.auth_token_service.create_token(&user_session.user_id.to_uuid(), user_session).await?;

        Ok(SessionToken(session_token))
    }

    /*
    pub async fn logout(&self, session_token: &SessionToken) -> Result<()>{
        let user_session = self.get_user_from_session_token(&session_token).await?;
        let user_id = user_session.user_id;
        self.delete_session(&session_token, &user_id).await?;
        Ok(())
    }
    */

    pub async fn delete_session(&self, session_token: &SessionToken, _user_id: &UserId) -> Result<()>{
        self.auth_token_service.delete_token(&session_token.to_uuid()).await?;

        Ok(())
    }

    pub async fn delete_all_sessions(&self, user_id: &UserId) -> Result<()>{
        self.auth_token_service.clear_tokens(&user_id.to_uuid()).await?;

        Ok(())
    }

    pub async fn verify_all_sessions(&self, user_id: &UserId) -> Result<()>{
        /*
            after the user has verified that their email address is valid, we should verify all of their sessions
         */

        let sessions = self.auth_token_service.get_tokens(&user_id.to_uuid()).await?;

        for(session_token, maybe_session) in sessions{
            match maybe_session{
                None => {},
                Some(mut user_session) => {
                    user_session.is_verified = true;
                    self.auth_token_service.update_token(&session_token, user_session).await?;
                }
            }
        }

        Ok(())
    }

    pub async fn verify_ip_all_sessions(&self, user_id: &UserId, ip: &IpAddr) -> Result<()>{
        /*
            after the user has verified that their email address is valid, we should verify all of their sessions
         */
        let sessions = self.auth_token_service.get_tokens(&user_id.to_uuid()).await?;

        for(session_token, maybe_session) in sessions{
            match maybe_session{
                None => {},
                Some(mut user_session) => {
                    if user_session.ip.to_string() == ip.to_string() {
                        user_session.is_known_ip = true;
                        self.auth_token_service.update_token(&session_token, user_session).await?;
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn get_user_from_session_token(&self, session_token: &SessionToken) -> Result<Option<UserSession>>{
        let user_session = self.auth_token_service.get_token(&session_token.to_uuid()).await?;

        Ok(user_session)
    }

}