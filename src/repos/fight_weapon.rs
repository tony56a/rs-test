use crate::constants::AWS_RESOURCE_REGION;
use crate::models::fight_weapon::FightWeapon;
use async_trait::async_trait;
use dynomite::retry::{Policy, RetryingDynamoDb};
use dynomite::{
    dynamodb::{
        DeleteItemInput, DynamoDb, DynamoDbClient, GetItemInput, PutItemError, PutItemInput,
    },
    Attribute, AttributeValue, FromAttributes, Retries,
};
use rusoto_core::RusotoError;
use std::collections::HashMap;

#[async_trait]
pub trait FightWeaponRepository {
    async fn create_fight_weapon(&self, fight_weapon: &FightWeapon);
    async fn update_fight_weapon(
        &self,
        weapon_name: &str,
        server_name: &str,
        fight_weapon: &FightWeapon,
    ) -> Result<FightWeapon, RusotoError<PutItemError>>;
    async fn get_fight_weapon(&self, weapon_name: &str, server_name: &str) -> Option<FightWeapon>;
    async fn delete_fight_weapon(&self, weapon_name: &str, server_name: &str);
}

pub struct FightWeaponDDBRepository {
    pub client: RetryingDynamoDb<DynamoDbClient>,
    table_name: String,
}

impl FightWeaponDDBRepository {
    pub fn new(table_name: &str) -> FightWeaponDDBRepository {
        FightWeaponDDBRepository {
            client: DynamoDbClient::new(AWS_RESOURCE_REGION).with_retries(Policy::default()),
            table_name: table_name.into(),
        }
    }

    pub fn new_with_client(
        client: &RetryingDynamoDb<DynamoDbClient>,
        table_name: &str,
    ) -> FightWeaponDDBRepository {
        FightWeaponDDBRepository {
            client: client.clone(),
            table_name: table_name.into(),
        }
    }
}

#[async_trait]
impl FightWeaponRepository for FightWeaponDDBRepository {
    async fn create_fight_weapon(&self, fight_weapon: &FightWeapon) {
        let fight_weapons_attributes = fight_weapon.clone().into();
        let result = self
            .client
            .put_item(PutItemInput {
                item: fight_weapons_attributes,
                table_name: self.table_name.clone(),
                ..PutItemInput::default()
            })
            .await;

        match result {
            Ok(_) => {}
            Err(e) => {
                println!("Error while writing result to DynamoDB: {:?}", e);
            }
        };
    }

    async fn update_fight_weapon(
        &self,
        weapon_name: &str,
        server_name: &str,
        fight_weapon: &FightWeapon,
    ) -> Result<FightWeapon, RusotoError<PutItemError>> {
        let fight_weapons_attributes = fight_weapon.clone().into();
        let result = self
            .client
            .put_item(PutItemInput {
                condition_expression: Some(
                    "attribute_exists(name) AND attribute_exists(server_name)".into(),
                ),
                item: fight_weapons_attributes,
                table_name: self.table_name.clone(),
                ..PutItemInput::default()
            })
            .await;

        match result {
            Ok(_) => Ok(fight_weapon.clone()),
            Err(e) => {
                println!("Error while writing result to DynamoDB: {:?}", e);
                Err(e)
            }
        }
    }

    async fn get_fight_weapon(&self, weapon_name: &str, server_name: &str) -> Option<FightWeapon> {
        let mut key_mapping: HashMap<String, AttributeValue> = HashMap::new();
        key_mapping.insert("name".into(), weapon_name.to_string().into_attr());
        key_mapping.insert("server_name".into(), server_name.to_string().into_attr());

        let result = self
            .client
            .get_item(GetItemInput {
                key: key_mapping,
                table_name: self.table_name.clone(),
                consistent_read: Some(true),
                ..GetItemInput::default()
            })
            .await;

        match result {
            Ok(response) => response.item.map_or_else(
                || None,
                |mut value| FightWeapon::from_attrs(&mut value).ok(),
            ),
            Err(e) => {
                println!("Error while getting results from DynamoDB: {:?}", e);
                None
            }
        }
    }

    async fn delete_fight_weapon(&self, weapon_name: &str, server_name: &str) {
        let mut key_mapping: HashMap<String, AttributeValue> = HashMap::new();
        key_mapping.insert("name".into(), weapon_name.to_string().into_attr());
        key_mapping.insert("server_name".into(), server_name.to_string().into_attr());

        let result = self
            .client
            .delete_item(DeleteItemInput {
                key: key_mapping,
                table_name: self.table_name.clone(),
                ..DeleteItemInput::default()
            })
            .await;

        match result {
            Ok(_) => {}
            Err(e) => {
                println!("Error while deleting weapon from DynamoDB: {:?}", e);
            }
        }
    }
}
