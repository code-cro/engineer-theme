#![allow(dead_code)]
use crate::core::traits::base::{Entity, Gateway};
use crate::core::traits::data_source::TableGateway;
use crate::core::traits::object_relational::structural::IdentityField;
use crate::core::types::sqlite3::Params;
use crate::models::User;
use rusqlite::{Connection, NO_PARAMS};
use std::include_str;

/// implement layer super-type marker trait
impl Entity for User {}

impl IdentityField for User {
    type IdType = String;

    fn id(self: &Self) -> &Self::IdType {
        &self.uuid
    }
}

pub struct UserTableGateway<'a> {
    connection: &'a Connection,
}

impl<'a> Gateway<'a> for UserTableGateway<'a> {
    type Connection = Connection;

    fn init(connection: &'a Self::Connection) -> UserTableGateway {
        UserTableGateway { connection }
    }
}

impl<'a> TableGateway<'a> for UserTableGateway<'a> {
    type Model = User;
    type Params = Params;

    fn insert(self: &Self, params: &Self::Params) -> bool {
        let mut sql_statement = self
            .connection
            .prepare(include_str!("../sql/user/insert.sql"))
            .unwrap();

        sql_statement
            .execute(&[
                params.get("uuid").unwrap(),
                params.get("username").unwrap(),
                params.get("password").unwrap(),
            ])
            .is_ok()
    }

    fn find(self: &Self, id: &str) -> Option<Self::Model> {
        let mut sql_statement = self
            .connection
            .prepare(include_str!("../sql/user/find.sql"))
            .unwrap();
        match sql_statement.query_row(&[id], |row| {
            Ok(User {
                uuid: row.get(0).unwrap(),
                username: row.get(1).unwrap(),
                password: row.get(2).unwrap(),
                inserted_at: Some(row.get(3).unwrap()),
            })
        }) {
            Ok(user) => Some(user),
            Err(_) => None,
        }
    }

    fn update(self: &Self, params: &Self::Params) -> bool {
        self.connection
            .execute(
                include_str!("../sql/user/update.sql"),
                &[
                    &params.get("username").unwrap(),
                    &params.get("password").unwrap(),
                    &params.get("uuid").unwrap(),
                ],
            )
            .is_ok()
    }

    fn delete(self: &Self, id: &str) -> bool {
        self.connection
            .execute(include_str!("../sql/user/delete.sql"), &[id])
            .is_ok()
    }
}

// this should be done in a different way, by making more smart "find" method
impl<'a> UserTableGateway<'a> {
    pub fn find_all(self: &Self) -> Option<Vec<User>> {
        match self
            .connection
            .prepare(include_str!("../sql/user/find_all.sql"))
            .unwrap()
            .query_map(NO_PARAMS, |row| {
                Ok(User {
                    uuid: row.get(0).unwrap(),
                    username: row.get(1).unwrap(),
                    password: row.get(2).unwrap(),
                    inserted_at: Some(row.get(3).unwrap()),
                })
            }) {
            Ok(result) => {
                let mut users: Vec<User> = Vec::new();

                for i in result {
                    if let Ok(user) = i {
                        users.push(user)
                    }
                }

                Some(users)
            }
            Err(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::types::Value;

    #[test]
    fn test_user_gateway() {
        let connection = Connection::open_in_memory().unwrap();
        assert!(connection
            .execute_batch(include_str!("../sql/user/__create__.sql"))
            .is_ok());

        let user_gateway = UserTableGateway::init(&connection);

        let mut insert_params = Params::new();

        let user = User::new("Josip".to_owned(), "1q2w3e4r".to_owned());

        insert_params.insert("username".to_owned(), Value::Text(user.username));
        insert_params.insert("password".to_owned(), Value::Text(user.password));
        insert_params.insert("uuid".to_owned(), Value::Text(user.uuid));

        assert!(user_gateway.insert(&insert_params));

        let users = user_gateway.find_all();

        if let Some(user_list) = users {
            assert_eq!(user_list.capacity(), 1);
            let user = user_gateway.find(&user_list[0].uuid);

            if let Some(usr) = user {
                assert_eq!(usr.uuid, user_list[0].uuid);
                assert_eq!(usr.username, user_list[0].username);
                assert_eq!(usr.password, user_list[0].password);
            }
        }

        assert!(connection
            .execute_batch(include_str!("../sql/user/__delete__.sql"))
            .is_ok());
    }
}
