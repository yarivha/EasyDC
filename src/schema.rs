use sqlx::sqlite::SqlitePool;

pub async fn init_tables(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let schema = r#"
        CREATE TABLE IF NOT EXISTS web_users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS dc_servers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            display_name TEXT NOT NULL,
            ldap_url TEXT NOT NULL,
            bind_dn TEXT NOT NULL,
            bind_password TEXT NOT NULL,
            skip_tls BOOLEAN DEFAULT 0
        );
    "#;

    // Execute the schema string against the database
    sqlx::query(schema).execute(pool).await?;
    
    println!("Database schema initialized successfully.");
    Ok(())
}
