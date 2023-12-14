use serenity::{
    all::{Message, ReactionType, User},
    client::Context,
};
use std::{collections::HashMap, time::Duration};

pub struct ReactionCollector<'a, T>
where
    T: ActionEnumTrait + Clone,
{
    pub(crate) action_map: HashMap<&'a str, T>,
}

impl<'a, T> ReactionCollector<'a, T>
where
    T: ActionEnumTrait + Clone,
{
    pub(crate) fn create(map: HashMap<&'a str, T>) -> Self {
        let x = ReactionCollector { action_map: map };
        x
    }

    pub(crate) fn add_action(mut self, s: &'a str, action: T) -> Self {
        self.action_map.insert(s, action);
        self
    }

    pub(crate) async fn wait_reaction(self, user: &User, msg: Message, ctx: &Context) -> Option<T> {
        for (reaction, _) in &self.action_map {
            let _ = msg
                .react(ctx, ReactionType::Unicode(reaction.to_string()))
                .await;
        }

        let collector = msg
            .await_reaction(&ctx.shard)
            .timeout(Duration::from_secs(10_u64))
            .author_id(user.id);

        let reaction = collector.await;
        if let Some(ReactionType::Unicode(emoji)) = reaction.map(|x| x.emoji) {
            if let Some(action) = self.action_map.get(emoji.as_str()).cloned() {
                return Some(action);
            }
        }
        None
    }
}

pub(crate) trait ActionEnumTrait {
    fn fallback_action() -> Self;
}
