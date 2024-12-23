use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use rusqlite::{params, Connection};
use std::process::Command;

#[derive(Parser)]
#[command(name = "gitsu")]
#[command(about = "Git User management")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Create {
        name: String,
        email: String,
    },
    List,
    Su {
        name: String,
    },
    Delete {
        name: String,
    },
}

fn init_db() -> Result<Connection> {
    let conn = Connection::open("git_users.sqlite")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            name TEXT PRIMARY KEY,
            email TEXT NOT NULL
        )",
        [],
    )?;
    Ok(conn)
}

fn create_user(conn: &Connection, name: &str, email: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO users (name, email) VALUES (?1, ?2)",
        params![name, email],
    )?;
    println!("create user success: {} <{}>", name, email);
    Ok(())
}

fn switch_user(conn: &Connection, name: &str) -> Result<()> {
    let email: String = conn
        .query_row(
            "SELECT email FROM users WHERE name = ?1",
            params![name],
            |row| row.get(0),
        )
        .context("user not found")?;

    Command::new("git")
        .args(["config", "user.name", name])
        .status()
        .context("`git config user.name` failed")?;

    Command::new("git")
        .args(["config", "user.email", &email])
        .status()
        .context("`git config user.email` failed")?;

    println!("user switched: {} <{}>", name, email);
    Ok(())
}

fn list_users(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare("SELECT name, email FROM users")?;
    let users = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;

    println!("Users:");
    for user in users {
        let (name, email) = user?;
        println!("- {} <{}>", name, email);
    }
    Ok(())
}

fn delete_user(conn: &Connection, name: &str) -> Result<()> {
    let rows = conn.execute("DELETE FROM users WHERE name = ?1", params![name])?;
    if rows > 0 {
        println!("user deleted: {} <{}>", name, email);
    } else {
        println!("user not found: {}", name);
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let conn = init_db()?;

    match cli.command {
        Commands::Create { name, email } => create_user(&conn, &name, &email)?,
        Commands::List => list_users(&conn)?,
        Commands::Su { name } => switch_user(&conn, &name)?,
        Commands::Delete { name } => delete_user(&conn, &name)?,
    }

    Ok(())
}
