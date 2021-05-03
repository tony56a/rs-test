use crate::constants::AWS_RESOURCE_REGION;
use crate::models::fight_user::FightUser;
use async_trait::async_trait;
use dynomite::retry::{Policy, RetryingDynamoDb};
use dynomite::{
    dynamodb::{
        DeleteItemInput, DynamoDb, DynamoDbClient, GetItemInput, PutItemError, PutItemInput,
    },
    Attribute, AttributeError, AttributeValue, Attributes, FromAttributes, Retries,
};
use rusoto_core::RusotoError;
use std::collections::HashMap;

#[async_trait]
pub trait FightUserRepository {
    async fn create_fight_user(&self, fight_user: &FightUser);
    async fn update_fight_user(
        &self,
        user_id: &str,
        server_name: &str,
        fight_user: &FightUser,
    ) -> Result<FightUser, RusotoError<PutItemError>>;
    async fn get_fight_user(&self, user_id: &str, server_name: &str) -> Option<FightUser>;
    async fn delete_fight_user(&self, user_id: &str, server_name: &str);
}

pub struct FightUserDDBRepository {
    pub client: RetryingDynamoDb<DynamoDbClient>,
    table_name: String,
}

impl FightUserDDBRepository {
    pub fn new(table_name: &str) -> FightUserDDBRepository {
        FightUserDDBRepository {
            client: DynamoDbClient::new(AWS_RESOURCE_REGION).with_retries(Policy::default()),
            table_name: table_name.into(),
        }
    }

    pub fn new_with_client(
        client: &RetryingDynamoDb<DynamoDbClient>,
        table_name: &str,
    ) -> FightUserDDBRepository {
        FightUserDDBRepository {
            client: client.clone(),
            table_name: table_name.into(),
        }
    }
}

#[async_trait]
impl FightUserRepository for FightUserDDBRepository {
    async fn create_fight_user(&self, fight_user: &FightUser) {
        let fight_user_attributes = fight_user.clone().into();
        let result = self
            .client
            .put_item(PutItemInput {
                item: fight_user_attributes,
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

    async fn update_fight_user(
        &self,
        user_id: &str,
        server_id: &str,
        fight_user: &FightUser,
    ) -> Result<FightUser, RusotoError<PutItemError>> {
        let fight_user_attributes = fight_user.clone().into();
        let result = self
            .client
            .put_item(PutItemInput {
                condition_expression: Some(
                    "attribute_exists(user_id) AND attribute_exists(server_name)".into(),
                ),
                item: fight_user_attributes,
                table_name: self.table_name.clone(),
                ..PutItemInput::default()
            })
            .await;

        match result {
            Ok(_) => Ok(fight_user.clone()),
            Err(e) => {
                println!("Error while writing result to DynamoDB: {:?}", e);
                Err(e)
            }
        }
    }

    async fn get_fight_user(&self, user_id: &str, server_name: &str) -> Option<FightUser> {
        let mut key_mapping: HashMap<String, AttributeValue> = HashMap::new();
        key_mapping.insert("user_id".into(), user_id.to_string().into_attr());
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
            Ok(response) => response
                .item
                .map_or_else(|| None, |mut value| FightUser::from_attrs(&mut value).ok()),
            Err(e) => {
                println!("Error while getting results from DynamoDB: {:?}", e);
                None
            }
        }
    }

    async fn delete_fight_user(&self, user_id: &str, server_name: &str) {
        let mut key_mapping: HashMap<String, AttributeValue> = HashMap::new();
        key_mapping.insert("user_id".into(), user_id.to_string().into_attr());
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
                println!("Error while deleting user from DynamoDB: {:?}", e);
            }
        }
    }
}
