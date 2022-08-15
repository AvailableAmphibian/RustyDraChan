use poise::{FrameworkContext, Event, Event::ReactionAdd};
use poise::serenity_prelude::{Context, Emoji, Reaction, ReactionType::{Custom, Unicode}, RoleId};
use super::super::helper::*;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, Condition, DatabaseConnection,
    QueryFilter, EntityTrait, ActiveModelTrait
};

use core::default::Default as BaseDefault;
use poise::futures_util::SinkExt;
use crate::entity::{rdc_reactionrole, prelude::RdcReactionrole};


// fetch one item
// let rr: Option<entity::rdc_reactionrole::Model> = RdcReactionrole::find_by_id(1).one(&db).await.expect("Grr");
// if let None = rr {
//     println!("Rn, that's okay")
// }

// insert one item
async fn insert_role_in_db(guild_id:u64, channel_id:u64, message_id:u64, role_id:u64, emoji: String) {
    // let emoji_as_str:str;
    //
    // if let Custom { id } = emoji {
    //
    // }

    // let reaction_role = rdc_reactionrole::ActiveModel {
    //     guild_id: Set(guild_id as i64),
    //     channel_id: Set(channel_id as i64),
    //     message_id: Set(message_id as i64),
    //     role_id: Set(role_id as i64),
    //     emoji: Set("❤️".to_owned()),
    //     ..BaseDefault::default()
    // };
    //
    // let reaction_role: rdc_reactionrole::Model = reaction_role.insert(&db).await.expect("Duh you can't insert here");

}


/// Displays your or another user's account creation date
pub async fn rr(
    context: &Context,
    event: &Event<'_>,
    framework: FrameworkContext<'_, Data, Error >)
    -> Result<(),Error>{
    let db = &framework.user_data.db;

    if let ReactionAdd { add_reaction} = event {
        let user_id = add_reaction.user_id.unwrap().0;
        let gid = match add_reaction.guild_id {
            Some(gid) => gid.0,
            _ => 0
        };
        let message_id = add_reaction.message_id.0;
        let channel_id = add_reaction.channel_id.0;
        let emoji = match &add_reaction.emoji {
            Custom { id, .. } => id.to_string(),
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
