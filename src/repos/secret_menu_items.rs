use crate::constants::SECRET_MENU_DB_PATH;
use crate::models::secret_menu_item::SecretMenuItem;
use async_trait::async_trait;
use rusqlite::{Connection, Error};

pub trait SecretMenuItemRepository {
    fn get_random_secret_item(&self) -> Option<SecretMenuItem>;
}

pub struct SecretMenuItemSqliteRepository {
    conn: Connection,
}

impl SecretMenuItemSqliteRepository {
    pub fn new() -> SecretMenuItemSqliteRepository {
        let conn =
            Connection::open(SECRET_MENU_DB_PATH).expect("SQLite connection couldn't be opened");
        SecretMenuItemSqliteRepository { conn }
    }
}

impl SecretMenuItemRepository for SecretMenuItemSqliteRepository {
    fn get_random_secret_item(&self) -> Option<SecretMenuItem> {

        let mut stmt = self.conn.prepare(
            "SELECT name, resturant, link \
                FROM secrets \
                ORDER BY random() \
                LIMIT 1 ",).ok()?;
        let mut rows_iter = stmt.query_map([], |row| {
            Ok(SecretMenuItem {
                name: row.get(0)?,
                restaurant: row.get(1)?,
                link: row.get(2)?,
            })
        }).ok()?;

        let row = rows_iter.next();
        match row.unwrap() {
            Ok(entry) => Some(entry),
            Err(_) => Option::None,
        }
    }
}
