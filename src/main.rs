mod commands;

use poise::{serenity_prelude as serenity};
use crate::commands::*;

pub mod helper {
    pub type Error = Box<dyn std::error::Error + Send + Sync>;
    pub type Context<'a> = poise::Context<'a, Data, Error>;
    // User data, which is stored and accessible in all command invocations
    pub struct Data {}
}

use crate::helper::*;

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

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![age(), register()],
            listener: |_, event, framework, _| {
                Box::pin(reaction_role::rr(/*Context::from(ctx), */event, framework))
            },
            ..Default::default()
        })
        .token(token) // Memo for fish : `set -gx DISCORD_TOKEN`
        .intents(serenity::GatewayIntents::non_privileged())
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(Data {}) }));


    println!("Built the framework");

    framework.run().await.unwrap();


}
