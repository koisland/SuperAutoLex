use anyhow::bail;

use crate::{
    token::{
        actions::ActionType, attribute::EntityType, logic::LogicType, numeric::NumericType,
        position::PositionType, target::TargetType, types::TokenType, SAPTokens,
    },
    trigger::EffectTrigger,
};

#[derive(Debug, Default)]
pub struct Effect<'src> {
    /// Effect trigger.
    pub trigger: Option<EffectTrigger<'src>>,
    /// Target of the effect.
    pub target: Option<TargetType>,
    /// Affected entities.
    pub entities: Option<Vec<EntityType<'src>>>,
    /// Position of target to affect.
    pub position: Option<PositionType>,
    /// Action to take.
    pub action: Option<ActionType>,
    /// Number of uses of effect per trigger.
    /// * `None` indicates unlimited uses.
    pub uses: Option<usize>,
    /// If the effect is temporary or not.
    pub temp: bool,
}

impl<'src> Effect<'src> {
    fn new(trigger: EffectTrigger, tokens: &SAPTokens) -> anyhow::Result<Self> {
        let mut tokens = tokens.iter().peekable();
        let mut effect = Effect::default();

        while let Some(token) = tokens.next() {
            match &token.ttype {
                TokenType::Numeric(NumericType::Most) => {
                    // Check for most health/attack.
                    let next_token_is_stats_attr = tokens.next_if(|token| {
                        matches!(
                            token.ttype,
                            TokenType::Entity(EntityType::Attack(None))
                                | TokenType::Entity(EntityType::Health(None))
                        )
                    });
                    match next_token_is_stats_attr.map(|token| &token.ttype) {
                        Some(TokenType::Entity(EntityType::Attack(None))) => {
                            effect.position = Some(PositionType::Strongest)
                        }
                        Some(TokenType::Entity(EntityType::Health(None))) => {
                            effect.position = Some(PositionType::Healthiest)
                        }
                        _ => {}
                    }
                }
                TokenType::Numeric(NumericType::Least) => todo!(),
                TokenType::Numeric(_) => todo!(),
                // Get petname and foodname from token text.
                TokenType::Entity(EntityType::Pet {
                    number: None,
                    name: Some(name),
                }) => todo!(),
                TokenType::Entity(EntityType::Food {
                    number: None,
                    name: Some(name),
                }) => todo!(),
                TokenType::Entity(_) => todo!(),
                TokenType::End => todo!(),
                TokenType::Position(_) => todo!(),
                TokenType::Target(_) => todo!(),
                TokenType::Logic(_) => todo!(),
                TokenType::Action(action) => effect.action = Some(*action),
            }
        }

        Ok(effect)
    }
}

#[cfg(test)]
mod test {
    use crate::{trigger::EffectTrigger, SAPText};

    use super::Effect;

    #[test]
    fn test_interpret_effect() {
        let trigger_txt = SAPText::new("Enemy summoned");
        let effect_txt =
            // SAPText::new("Deal 100% attack damage to the most healthy enemy and itself.");
            SAPText::new("Summon one Loyal Chinchilla.");

        let triggers: Vec<EffectTrigger> = trigger_txt.tokenize().unwrap().try_into().unwrap();
        let effect_tokens = effect_txt.tokenize().unwrap();

        for token in effect_tokens.iter() {
            println!("{token}")
        }

        // for trigger in triggers {
        //     let effect = Effect::new(trigger, &effect_tokens).unwrap();
        //     println!("{effect:?}")
        // }
    }
}
