use poise::{FrameworkContext, Event, Event::ReactionAdd};
use poise::serenity_prelude::{Channel, Context, ReactionType::{Custom, Unicode}, ReactionType, Role, RoleId};
use super::super::helper::*;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, Condition, DatabaseConnection,
    QueryFilter, EntityTrait, ActiveModelTrait
};

use core::default::Default as BaseDefault;
use crate::entity::{rdc_reactionrole, prelude::RdcReactionrole};
use crate::helper;


// fetch one item
// let rr: Option<entity::rdc_reactionrole::Model> = RdcReactionrole::find_by_id(1).one(&db).await.expect("Grr");
// if let None = rr {
//     println!("Rn, that's okay")
// }

// insert one item
async fn insert_role_in_db(db: &DatabaseConnection, guild_id:u64, channel_id:u64, message_id:u64, role_id:u64, emoji: String) {
    let reaction_role = rdc_reactionrole::ActiveModel {
        guild_id: Set(guild_id as i64),
        channel_id: Set(channel_id as i64),
        message_id: Set(message_id as i64),
        role_id: Set(role_id as i64),
        emoji: Set(emoji.to_owned()),
        ..BaseDefault::default()
    };

    let _reaction_role: rdc_reactionrole::Model = reaction_role.insert(db).await.expect("Duh you can't insert here");
}


/// Give or remove a role according to a reaction
pub async fn rr(
    context: &Context,
    event: &Event<'_>,
    framework: FrameworkContext<'_, Data, Error >)
    -> Result<(),Error>{
    let db = &framework.user_data.db;

    if let ReactionAdd { add_reaction} = event {
        let user_id = add_reaction.user_id.unwrap().0;

        if user_id == framework.bot_id.0 {
            return Ok(());
        }

        let gid = match add_reaction.guild_id {
            Some(gid) => gid.0,
            _ => 0
        };
        let message_id = add_reaction.message_id.0;
        let channel_id = add_reaction.channel_id.0;
        let emoji = match &add_reaction.emoji {
            Custom { id, name, .. } => format!("<:{}:{}>", name.as_ref().unwrap(), id.to_string()),
            Unicode(emote) => emote.to_string(),
            _ => panic!("Did not expect to get something else than Custom or Unicode !")
        };

        let conditions = Condition::all()
            .add(rdc_reactionrole::Column::GuildId.eq(gid))
            .add(rdc_reactionrole::Column::MessageId.eq(message_id))
            .add(rdc_reactionrole::Column::ChannelId.eq(channel_id))
            .add(rdc_reactionrole::Column::Emoji.eq(emoji));

        let rr: Option<rdc_reactionrole::Model> = RdcReactionrole::find()
            .filter(conditions)
            .one(db).await.expect("Grr");

        let http = &context.http;
        match rr {
            Some(reac) => {
                let member = &add_reaction.member;
                let member = member.as_ref().unwrap();
                let role_id = reac.role_id as u64;
                println!("User {} in {} asked for {}", user_id, gid, role_id);
                if member.roles.contains(&RoleId(role_id)){
                    println!("--- Removing role {} ---", role_id);
                    http.remove_member_role(gid, user_id, role_id,None).await?;
                } else {
                    println!("--- Giving role {} ---", role_id);
                    http.add_member_role(gid, user_id, role_id, None).await?;
                }
                println!("Deleting user reaction");
                http.delete_reaction(channel_id, message_id, Some(user_id), &add_reaction.emoji).await?;

            },
            None => println!("No role to give !")
        }
    }
    Ok(())
}

/// Create a reaction role
#[poise::command(slash_command)]
pub async fn create_rr(
    ctx: helper::Context<'_>,
    #[description = "Selected channel"] channel: Channel,
    #[description = "Selected message"] message_id: String,
    #[description = "Selected role"] role: Role,
    #[description = "Selected reaction"] reaction: String
) -> Result<(), Error> {
    let msg = match unsafe_create_rr(ctx, channel, message_id, role, reaction).await {
        Ok(()) => String::from("Successfully created a reaction role !"),
        Err(error) => format!("An error occurred !\nError : {}", error)
    };
    ctx.say(msg).await?;
    Ok(())
}

async fn unsafe_create_rr(ctx: helper::Context<'_>, channel:Channel, message_id:String, role: Role, reaction:String) -> Result<(), Error> {
    let http = &ctx.discord().http;
    let channel_id = channel.id().0;
    let message_id = message_id.parse::<u64>()?;
    let reaction_type = ReactionType::try_from(reaction.as_ref())?;

    insert_role_in_db(&ctx.data().db, role.guild_id.0, channel_id, message_id, role.id.0, reaction).await;

    http.create_reaction(channel_id, message_id, &reaction_type).await?;

    Ok(())
}