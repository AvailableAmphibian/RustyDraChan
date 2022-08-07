use poise::{FrameworkContext, Event, Event::ReactionAdd};
use super::super::helper::*;

/// Displays your or another user's account creation date
pub async fn rr(
    event: &Event<'_>,
    _framework_ctx: FrameworkContext<'_, Data, Error >)
    -> Result<(),Error>{
    if let ReactionAdd { add_reaction} = event {
        println!("{:?}",add_reaction)
    }
    Ok(())
}
