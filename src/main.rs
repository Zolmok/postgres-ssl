use openssl::ssl::{SslConnector, SslFiletype, SslMethod};
use postgres_openssl::MakeTlsConnector;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct DatabaseConfig {
    client_cert_path: String,
    client_key_path: String,
    server_ca_path: String,
    host: String,
    dbname: String,
    user: String,
    password: String,
}

impl ::std::default::Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            client_cert_path: "".into(),
            client_key_path: "".into(),
            server_ca_path: "".into(),
            host: "".into(),
            dbname: "".into(),
            user: "".into(),
            password: "".into(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), tokio_postgres::Error> {
    let config: DatabaseConfig = match confy::load("postgres-ssl") {
        Ok(value) => value,
        Err(error) => panic!("confy-load error: {}", error),
    };

    let client_cert_path = config.client_cert_path;
    let client_key_path = config.client_key_path;
    let server_ca_path = config.server_ca_path;
    let host = config.host;
    let dbname = config.dbname;
    let user = config.user;
    let password = config.password;
    let connection_string = format!(
        "sslmode=require host={} dbname={} user={} password={}",
        host, dbname, user, password
    );

    let mut builder = match SslConnector::builder(SslMethod::tls()) {
        Ok(value) => value,
        Err(error) => panic!("connector error: {}", error),
    };

    if let Err(error) = builder.set_certificate_chain_file(&client_cert_path) {
        eprintln!("set_certificate_file: {}", error);
    }
    if let Err(error) = builder.set_private_key_file(&client_key_path, SslFiletype::PEM) {
        eprintln!("set_client_key_file: {}", error);
    }
    if let Err(error) = builder.set_ca_file(&server_ca_path) {
        eprintln!("set_ca_file: {}", error);
    }

    let mut connector = MakeTlsConnector::new(builder.build());

    connector.set_callback(|config, _| {
        config.set_verify_hostname(false);

        Ok(())
    });

    let connect = tokio_postgres::connect(&connection_string, connector);

    let (client, connection) = match connect.await {
        Ok(value) => value,
        Err(error) => panic!("connect error: {}", error),
    };

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(error) = connection.await {
            eprintln!("connection error: {}", error);
        }

        println!("client:{:?}", client);
    });

    Ok(())
}
