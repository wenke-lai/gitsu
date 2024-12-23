use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use rusqlite::{params, Connection};
use std::process::Command;

#[derive(Parser)]
#[command(name = "git_user_manager")]
#[command(about = "管理多個 Git 使用者設定")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 創建新的 Git 使用者
    Create {
        /// 使用者名稱
        name: String,
        /// 電子郵件
        email: String,
    },
    /// 切換到指定的 Git 使用者
    Su {
        /// 使用者名稱
        name: String,
    },
    /// 列出所有 Git 使用者
    List,
    /// 刪除指定的 Git 使用者
    Delete {
        /// 使用者名稱
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
    println!("使用者建立成功: {} ({})", name, email);
    Ok(())
}

fn switch_user(conn: &Connection, name: &str) -> Result<()> {
    let email: String = conn
        .query_row(
            "SELECT email FROM users WHERE name = ?1",
            params![name],
            |row| row.get(0),
        )
        .context("找不到指定的使用者")?;

    Command::new("git")
        .args(["config", "user.name", name])
        .status()
        .context("無法設定 git user.name")?;

    Command::new("git")
        .args(["config", "user.email", &email])
        .status()
        .context("無法設定 git user.email")?;

    println!("已切換到使用者: {} ({})", name, email);
    Ok(())
}

fn list_users(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare("SELECT name, email FROM users")?;
    let users = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;

    println!("已儲存的使用者：");
    for user in users {
        let (name, email) = user?;
        println!("- {} ({})", name, email);
    }
    Ok(())
}

fn delete_user(conn: &Connection, name: &str) -> Result<()> {
    let rows = conn.execute("DELETE FROM users WHERE name = ?1", params![name])?;
    if rows > 0 {
        println!("已刪除使用者: {}", name);
    } else {
        println!("找不到使用者: {}", name);
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let conn = init_db()?;

    match cli.command {
        Commands::Create { name, email } => create_user(&conn, &name, &email)?,
        Commands::Su { name } => switch_user(&conn, &name)?,
        Commands::List => list_users(&conn)?,
        Commands::Delete { name } => delete_user(&conn, &name)?,
    }

    Ok(())
}
