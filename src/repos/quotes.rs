use crate::constants::AWS_RESOURCE_REGION;
use crate::models::user_quote::UserQuote;
use async_trait::async_trait;
use dynomite::retry::{Policy, RetryingDynamoDb};
use dynomite::{
    dynamodb::{DeleteItemInput, DynamoDb, DynamoDbClient, PutItemInput, QueryInput},
    Attribute, AttributeValue, FromAttributes, Retries,
};
use rand::prelude::SliceRandom;
use serenity::model::id::{ChannelId, MessageId, UserId};
use std::collections::HashMap;
use uuid::Uuid;

#[async_trait]
pub trait UserQuoteRepository {
    async fn create_user_quote(&self, fight_weapon: &UserQuote);
    async fn get_quote_by_id(
        &self,
        message_id: MessageId,
        channel_id: ChannelId,
    ) -> Option<UserQuote>;
    async fn get_random_quote(&self, server_name: &str) -> Option<UserQuote>;
    async fn get_quotes_for_user(&self, user_id: UserId) -> Vec<UserQuote>;
    async fn delete_quote(&self, message_id: MessageId, channel_id: ChannelId, channel_name: &str);
}

pub struct UserQuoteDDBRepository {
    pub client: RetryingDynamoDb<DynamoDbClient>,
    table_name: String,
}

impl UserQuoteDDBRepository {
    pub fn new(table_name: &str) -> UserQuoteDDBRepository {
        UserQuoteDDBRepository {
            client: DynamoDbClient::new(AWS_RESOURCE_REGION).with_retries(Policy::default()),
            table_name: table_name.into(),
        }
    }

    pub fn new_with_client(
        client: &RetryingDynamoDb<DynamoDbClient>,
        table_name: &str,
    ) -> UserQuoteDDBRepository {
        UserQuoteDDBRepository {
            client: client.clone(),
            table_name: table_name.into(),
        }
    }
}

#[async_trait]
impl UserQuoteRepository for UserQuoteDDBRepository {
    async fn create_user_quote(&self, user_quote: &UserQuote) {
        let quote_attributes = user_quote.clone().into();
        let result = self
            .client
            .put_item(PutItemInput {
                item: quote_attributes,
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

    async fn get_quote_by_id(
        &self,
        message_id: MessageId,
        channel_id: ChannelId,
    ) -> Option<UserQuote> {
        let mut key_mapping: HashMap<String, AttributeValue> = HashMap::new();
        key_mapping.insert(":channel_id".into(), channel_id.to_string().into_attr());
        key_mapping.insert(":message_id".into(), message_id.to_string().into_attr());

        let query = self
            .client
            .query(QueryInput {
                expression_attribute_values: Some(key_mapping.clone()),
                key_condition_expression: Some(
                    "channel_id = :channel_id AND message_id = :message_id".to_string(),
                ),
                limit: Some(1),
                table_name: self.table_name.clone(),
                index_name: Some("ChannelGSI".to_string()),
                ..QueryInput::default()
            })
            .await;

        let mut result = match query {
            Ok(result) => result.items.unwrap(),
            Err(e) => {
                println!("Error while getting value from DynamoDB: {:?}", e);
                return None;
            }
        };

        if result.is_empty() {
            return None;
        }
        return UserQuote::from_attrs(&mut result[0]).ok();
    }

    async fn get_random_quote(&self, server_name: &str) -> Option<UserQuote> {
        let random_key = Uuid::new_v4().to_hyphenated().to_string();
        let mut key_mapping: HashMap<String, AttributeValue> = HashMap::new();
        key_mapping.insert(":sort_id".into(), random_key.into_attr());
        key_mapping.insert(":server_name".into(), server_name.to_string().into_attr());

        let gt_query = self
            .client
            .query(QueryInput {
                expression_attribute_values: Some(key_mapping.clone()),
                key_condition_expression: Some(
                    "server_name = :server_name AND sort_id > :sort_id".to_string(),
                ),
                limit: Some(10),
                table_name: self.table_name.clone(),
                ..QueryInput::default()
            })
            .await;

        let lt_query = self
            .client
            .query(QueryInput {
                expression_attribute_values: Some(key_mapping),
                key_condition_expression: Some(
                    "server_name = :server_name AND sort_id < :sort_id".to_string(),
                ),
                limit: Some(10),
                table_name: self.table_name.clone(),
                ..QueryInput::default()
            })
            .await;

        let mut gt_result = match gt_query {
            Ok(result) => result.items.unwrap(),
            Err(e) => {
                println!("Error while getting value from DynamoDB: {:?}", e);
                return None;
            }
        };

        let mut lt_result = match lt_query {
            Ok(result) => result.items.unwrap(),
            Err(e) => {
                println!("Error while getting value from DynamoDB: {:?}", e);
                return None;
            }
        };

        gt_result.append(&mut lt_result);

        if gt_result.is_empty() {
            return None;
        }
        let mut final_results: Vec<UserQuote> = gt_result
            .iter_mut()
            .filter_map(|mapping| UserQuote::from_attrs(mapping).ok())
            .collect();

        final_results.sort();
        final_results.dedup();
        let retval = final_results
            .choose(&mut rand::thread_rng())
            .unwrap()
            .clone();
        Some(retval)
    }

    async fn get_quotes_for_user(&self, user_id: UserId) -> Vec<UserQuote> {
        let mut key_mapping: HashMap<String, AttributeValue> = HashMap::new();
        key_mapping.insert(":user_id".into(), user_id.to_string().into_attr());

        let query = self
            .client
            .query(QueryInput {
                expression_attribute_values: Some(key_mapping.clone()),
                key_condition_expression: Some("author_id = :user_id".to_string()),
                table_name: self.table_name.clone(),
                index_name: Some("AuthorGSI".to_string()),
                ..QueryInput::default()
            })
            .await;

        match query {
            Ok(result) => result
                .items
                .unwrap()
                .iter_mut()
                .filter_map(|entry| UserQuote::from_attrs(entry).ok())
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    async fn delete_quote(&self, message_id: MessageId, channel_id: ChannelId, server_name: &str) {
        let mut key_mapping: HashMap<String, AttributeValue> = HashMap::new();
        key_mapping.insert(":channel_id".into(), channel_id.to_string().into_attr());
        key_mapping.insert(":message_id".into(), message_id.to_string().into_attr());

        let query = self
            .client
            .query(QueryInput {
                expression_attribute_values: Some(key_mapping.clone()),
                key_condition_expression: Some(
                    "channel_id = :channel_id AND message_id = :message_id".to_string(),
                ),
                limit: Some(1),
                table_name: self.table_name.clone(),
                index_name: Some("ChannelGSI".to_string()),
                ..QueryInput::default()
            })
            .await;

        let mut result = match query {
            Ok(result) => result.items.unwrap(),
            Err(e) => {
                println!("Error while getting value from DynamoDB: {:?}", e);
                return;
            }
        };

        if result.is_empty() {
            return;
        }
        let query_result = UserQuote::from_attrs(&mut result[0]).unwrap();

        let mut delete_key_mapping: HashMap<String, AttributeValue> = HashMap::new();
        delete_key_mapping.insert("server_name".into(), server_name.to_string().into_attr());
        delete_key_mapping.insert("sort_id".into(), query_result.sort_id_key().into_attr());

        let delete_query = self
            .client
            .delete_item(DeleteItemInput {
                key: delete_key_mapping,
                table_name: self.table_name.clone(),
                ..DeleteItemInput::default()
            })
            .await;

        match delete_query {
            Ok(_) => {}
            Err(e) => {
                println!("Could not delete quote due to error: {}", e);
            }
        }
    }
}
