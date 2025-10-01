pub mod noop_auth_component;
pub mod add_component;

use miden_objects::{account::{Account, AccountCode, AccountStorage}, asset::AssetVault, Felt};

trait ToJson {
  fn to_json(&self) -> serde_json::Value;
}

trait FromJson {
  fn from_json(json: serde_json::Value) -> Self;
}

impl ToJson for AssetVault {
  fn to_json(&self) -> serde_json::Value {
    todo!()
  }
}

impl ToJson for AccountStorage {
  fn to_json(&self) -> serde_json::Value {
    todo!()
  }
}

impl ToJson for AccountCode {
  fn to_json(&self) -> serde_json::Value {
    todo!()
  }
}

impl ToJson for Felt {
  fn to_json(&self) -> serde_json::Value {
    todo!()
  }
}

impl ToJson for Account {
  fn to_json(&self) -> serde_json::Value {
    let parts = self.clone().into_parts();
    let json = serde_json::json!({
      "account_id": parts.0.to_hex(),
      "vault": parts.1.to_json(),
      "storage": parts.2.to_json(),
      "code": parts.3.to_json(),
      "nonce": parts.4.to_json(),
    });
    json
  }
}



pub fn account_to_json(account: Account) {
  let parts = account.into_parts();
  let json = serde_json::json!({
    "account_id": format!("{:?}", parts.0),
    "vault": format!("{:?}", parts.1),
    "storage": format!("{:?}", parts.2),
    "code": format!("{:?}", parts.3),
    "nonce": format!("{:?}", parts.4),
  });
  println!("{:?}", serde_json::to_string_pretty(&json).unwrap());
}

// pub fn deserialize_account(json: String) -> Account {
//   let parts: (AccountId, AssetVault, AccountStorage, AccountCode, Felt) = serde_json::from_str(&json).unwrap();
//   let account = Account::from_parts(
//     parts.0,
//     parts.1,
//     parts.2,
//     parts.3,
//     parts.4,
//   );
//   account
  
// }