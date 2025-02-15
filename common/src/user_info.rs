use crate::token_fields::{Audiences, Issuer, SubscriberId};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Return value of `/myinfo` api
pub struct UserInfo {
  /// username
  pub username: String,
  /// allowed apps, i.e, client_ids
  pub allowed_apps: Audiences,
  /// issuer specified by url like 'https://example.com/' for IdToken
  pub issuer: Issuer,
  /// subscriber id generated by the token server
  pub subscriber_id: SubscriberId,
  /// is admin or not
  pub is_admin: bool,
}
