//ref: |https://docs.rs/mysql/latest/mysql/#example|

use std::error::Error;

use mysql::{params, prelude::Queryable, OptsBuilder, Pool, PooledConn};
#[cfg(test)]
use s3::creds::time::PrimitiveDateTime;

use super::color::Color;
use super::config::RDSConfig;

/*-------------------------------------*/

pub struct MySQL {
    connection: PooledConn,
    table_name: String,
}

impl MySQL {
    pub fn new(config: &RDSConfig) -> Result<Self, Box<dyn Error>> {
        let opts = OptsBuilder::new()
            .user(Some(config.user.to_string()))
            .pass(Some(config.password.to_string()))
            .ip_or_hostname(Some(config.host.to_string()))
            .tcp_port(config.port)
            .db_name(Some(config.database_name.to_string()));
        let pool = Pool::new(opts)?;
        let connection = pool.get_conn()?;

        let mut ret = Self {
            connection,
            table_name: config.table_name.to_string(),
        };
        ret.init()?;
        Ok(ret)
    }

    fn init(&mut self) -> Result<(), Box<dyn Error>> {
        let res = self.connection.query_drop(format!(
            r"CREATE TABLE IF NOT EXISTS {} (
                r           int       not null,
                g           int       not null,
                b           int       not null,
                inserted_at timestamp not null default current_timestamp
            )",
            &self.table_name
        ));
        if let Err(e) = res {
            Err(e.to_string().into())
        } else {
            Ok(())
        }
    }

    pub fn insert(&mut self, c: &Color) -> Result<(), Box<dyn Error>> {
        let res = self.connection.exec_drop(
            format!(
                r"INSERT INTO {} (r, g, b) VALUES (:r, :g, :b)",
                &self.table_name
            ),
            params! {
                "r" => c.r,
                "g" => c.g,
                "b" => c.b,
            },
        );
        if let Err(e) = res {
            Err(e.to_string().into())
        } else {
            Ok(())
        }
    }

    #[cfg(test)]
    fn select(&mut self) -> Result<Vec<Color>, Box<dyn Error>> {
        let res = self.connection.query_map(
            format!("SELECT * from {}", &self.table_name),
            |(r, g, b, _): (u8, u8, u8, PrimitiveDateTime)| Color::new(r, g, b),
        );
        if let Err(e) = res {
            Err(e.to_string().into())
        } else {
            Ok(res.unwrap())
        }
    }

    #[cfg(test)]
    pub fn select_by_color(&mut self, color: &Color) -> Result<Vec<Color>, Box<dyn Error>> {
        let res = self.connection.exec_map(
            format!(
                "SELECT * from {} where r = :r and g = :g and b = :b",
                &self.table_name
            ),
            params! {"r" => color.r, "g" => color.g, "b" => color.b},
            |(r, g, b, _): (u8, u8, u8, PrimitiveDateTime)| Color::new(r, g, b),
        );
        if let Err(e) = res {
            Err(e.to_string().into())
        } else {
            Ok(res.unwrap())
        }
    }
}

/*-------------------------------------*/

#[cfg(test)]
mod tests {
    use super::super::config::Config;
    use super::*;

    #[test]
    fn test01() -> Result<(), Box<dyn Error>> {
        let config = Config::new("./config.json");

        let db = MySQL::new(&config.rds);
        assert!(db.is_ok());
        let mut db = db.unwrap();

        let color = Color {
            r: 100,
            g: 50,
            b: 25,
        };

        let num_row = db.select_by_color(&color)?.len();

        let res = db.insert(&color);
        println!("{:?}", res);
        assert!(res.is_ok());

        assert_eq!(num_row + 1, db.select_by_color(&color)?.len());

        let res = db.select();
        println!("{:?}", res);
        assert!(res.is_ok());

        Ok(())
    }
}

/*-------------------------------------*/
