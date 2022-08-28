mod commands;
mod entity;

use std::time::Duration;
use poise::{serenity_prelude as serenity};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use crate::commands::reaction_role;


pub mod helper {
    use sea_orm::DatabaseConnection;

    pub type Error = Box<dyn std::error::Error + Send + Sync>;
    pub type Context<'a> = poise::Context<'a, Data, Error>;
    // User data, which is stored and accessible in all command invocations
    pub struct Data {
        pub(crate) db: DatabaseConnection
    }
}

use crate::helper::*;
use crate::reaction_role::create_rr;


/// Displays your or another user's account creation date
#[poise::command(slash_command)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(prefix_command, owners_only, hide_in_help)]
pub async fn register(ctx: Context<'_>, #[flag] global: bool) -> Result<(), Error> {
    poise::builtins::register_application_commands(ctx, global).await.map_err(Into::into)
}

async fn create_db_connection() -> Result<DatabaseConnection, sea_orm::DbErr> {
    let db_connector = std::env::var("DB_CONNECTOR")
        .expect("missing db connector; format : protocol://username:password@localhost/database");

    let mut opt = ConnectOptions::new(db_connector.to_owned());
    opt.max_connections(10)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8));

    Database::connect(opt).await
}

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");

    let db = create_db_connection().await.expect("Couldn't create a DatabaseConnection.");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![age(), register(), create_rr()],
            listener: |ctx, event, framework, _| {
                Box::pin(reaction_role::rr(ctx, event, framework))
            },
            ..Default::default()
        })
        .token(token) // Memo for fish : `set -gx DISCORD_TOKEN`
        .intents(serenity::GatewayIntents::non_privileged())
        .user_data_setup(move |_ctx, _ready, _framework| {
            Box::pin(async move {
                Ok(Data { db })
            })
        });


    println!("Built the framework");

    framework.run().await.unwrap();


}
