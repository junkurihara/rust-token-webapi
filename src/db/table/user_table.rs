use super::{super::entity::*, UserSearchKey, UserTable};
use crate::{constants::*, error::*};
use async_trait::async_trait;
use sqlx::sqlite::SqlitePool;
use std::convert::{From, TryInto};
use validator::Validate;

#[derive(Debug, Clone)]
pub struct SqliteUserTable {
  pool: SqlitePool,
}

impl SqliteUserTable {
  pub fn new(pool: SqlitePool) -> Self {
    Self { pool }
  }
}

#[async_trait]
impl UserTable for SqliteUserTable {
  async fn add(&self, user: User) -> Result<()> {
    let sql = format!(
      "insert into {} (username, subscriber_id, encoded_hash, is_admin) VALUES (?, ?, ?, ?)",
      USER_TABLE_NAME
    );
    let _res = sqlx::query(&sql)
      .bind(user.username.as_str())
      .bind(user.subscriber_id.as_str())
      .bind(user.encoded_hash.as_str())
      .bind(user.is_admin.into_string())
      .execute(&self.pool)
      .await?;
    Ok(())
  }

  async fn find_user<'a>(&self, user_search_key: UserSearchKey<'a>) -> Result<Option<User>> {
    let sql = match user_search_key {
      UserSearchKey::SubscriberId(sub_id) => format!(
        "select * from {} where subscriber_id='{}'",
        USER_TABLE_NAME,
        sub_id.as_str()
      ),
      UserSearchKey::Username(username) => {
        format!(
          "select * from {} where username='{}'",
          USER_TABLE_NAME,
          username.as_str()
        )
      }
    };
    let user_row_opt: Option<UserRow> = sqlx::query_as(&sql).fetch_optional(&self.pool).await?;
    if let Some(user_row) = user_row_opt {
      let user: User = user_row.try_into()?;
      Ok(Some(user))
    } else {
      Ok(None)
    }
  }
}

#[derive(Debug, sqlx::FromRow)]
struct UserRow {
  username: String,
  subscriber_id: String,
  encoded_hash: String,
  is_admin: String,
}

impl From<User> for UserRow {
  fn from(value: User) -> Self {
    Self {
      username: value.username.into_string(),
      subscriber_id: value.subscriber_id.into_string(),
      encoded_hash: value.encoded_hash.into_string(),
      is_admin: value.is_admin.get().to_string(),
    }
  }
}

impl TryInto<User> for UserRow {
  type Error = crate::error::Error;

  fn try_into(self) -> std::result::Result<User, Self::Error> {
    let x = User {
      username: Username::new(self.username)?,
      subscriber_id: SubscriberId::new(self.subscriber_id)?,
      encoded_hash: EncodedHash::new_from_raw(self.encoded_hash)?,
      is_admin: IsAdmin::new(self.is_admin.parse::<bool>()?)?,
    };
    x.username.validate()?;
    x.subscriber_id.validate()?;
    x.encoded_hash.validate()?;
    Ok(x)
  }
}
