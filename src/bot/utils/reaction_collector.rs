use serenity::{
    all::{Message, ReactionType, User},
    client::Context,
};
use std::{collections::HashMap, time::Duration};

pub struct ReactionCollector<'a, T>
where
    T: ActionEnumTrait + Clone,
{
    pub(crate) reactions: Vec<&'a str>,
    pub(crate) action_map: HashMap<&'a str, T>,
}

impl<'a, T> ReactionCollector<'a, T>
where
    T: ActionEnumTrait + Clone,
{
    pub(crate) fn create(vec: Vec<(&'a str, T)>) -> Self {
        let mut reactions: Vec<&'a str> = Vec::new();
        let mut action_map: HashMap<&'a str, T> = HashMap::new();

        for iter in 0..vec.len() {
            let (k, v) = vec[iter].clone();
            reactions.push(k);
            action_map.insert(k, v.clone());
        }
        let x = ReactionCollector {
            reactions,
            action_map,
        };
        x
    }

    pub(crate) fn add_action(mut self, s: &'a str, action: T, pos: Option<usize>) -> Self {
        if let Some(pos) = pos {
            self.reactions.insert(pos, s)
        } else {
            self.reactions.push(s)
        }
        self.action_map.insert(s, action);
        self
    }

    pub(crate) async fn wait_reaction(self, user: &User, msg: Message, ctx: &Context) -> Option<T> {
        for reaction in &self.reactions {
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
