use anyhow::bail;
use itertools::Itertools;

use crate::{
    token::{
        actions::ActionType, attribute::EntityType, logic::LogicType, numeric::NumericType,
        position::PositionType, target::TargetType, types::TokenType, SAPTokens, Token,
    },
    trigger::EffectTrigger,
};

#[derive(Debug, Default)]
pub struct Effect<'src> {
    /// Effect trigger.
    pub trigger: Option<EffectTrigger<'src>>,
    /// Secondary effect trigger for conditional effects.
    pub secondary_trigger: Option<EffectTrigger<'src>>,
    /// Target of the effect.
    pub target: Option<TargetType>,
    /// Affected entities.
    pub entities: Vec<EntityType<'src>>,
    /// Position of target to affect.
    pub position: Vec<PositionType>,
    /// Action to take.
    pub action: Option<ActionType>,
    /// Number of uses of effect per trigger.
    /// * `None` indicates unlimited uses.
    pub uses: Option<usize>,
    /// If the effect is temporary or not.
    pub temp: bool,
}

macro_rules! update_effect_max_min_stat_pos {
    ($tokens:ident, $effect:ident, atk = $attack_pos_type:expr, health = $health_pos_type:expr) => {
        // Check next token for most/least health/attack.
        match $tokens
            .next_if(|token| {
                matches!(
                    token.ttype,
                    TokenType::Entity(EntityType::Attack(None))
                        | TokenType::Entity(EntityType::Health(None))
                )
            })
            .map(|token| &token.ttype)
        {
            Some(TokenType::Entity(EntityType::Attack(None))) => {
                $effect.position.push($attack_pos_type)
            }
            Some(TokenType::Entity(EntityType::Health(None))) => {
                $effect.position.push($health_pos_type)
            }
            _ => {}
        }
    };
}
impl<'src> Effect<'src> {
    fn new(trigger: EffectTrigger, tokens: &'src SAPTokens) -> anyhow::Result<Self> {
        let mut token_window = tokens
            .iter()
            .tuple_windows::<(&'src Token, &'src Token, &'src Token)>()
            .peekable();
        let mut effects: Vec<Effect> = vec![];
        let mut effect = Effect::default();

        while let Some((token, token_2, token_3)) = token_window.next() {
            match &token.ttype {
                TokenType::Numeric(NumericType::Most) => {
                    // update_effect_max_min_stat_pos!(
                    //     tokens,
                    //     effect,
                    //     atk = PositionType::Strongest,
                    //     health = PositionType::Healthiest
                    // );
                }
                TokenType::Numeric(NumericType::Least) => {
                    // update_effect_max_min_stat_pos!(
                    //     tokens,
                    //     effect,
                    //     atk = PositionType::Weakest,
                    //     health = PositionType::Illest
                    // );
                }
                TokenType::Numeric(num) => {}
                TokenType::Entity(entity) => {
                    // Consume next token if damage attribute.
                    // This is attack/attack perc damage.
                    if matches!(entity, EntityType::Attack(_) | EntityType::AttackPercent(_)) {
                        if token_2.ttype == TokenType::Entity(EntityType::Damage(None)) {
                            token_window.next();
                        }
                    }
                    effect.entities.push(entity.clone())
                }
                TokenType::EndText => {}
                TokenType::Position(pos) => effect.position.push(*pos),
                TokenType::Target(target) => effect.target = Some(*target),
                // Construct secondary trigger if If if possible.
                TokenType::Logic(LogicType::If) => {}
                // Create new effect for each.
                TokenType::Logic(LogicType::ForEach) => {}
                TokenType::Logic(logic) => {}
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
    fn test_interpret_conditional_effect() {
        let trigger_txt = SAPText::new("End turn");
        let effect_txt =
            SAPText::new("If this has a level 3 friend, gain +1 attack and +2 health.");

        for token in effect_txt.tokenize().unwrap().iter() {
            println!("{token}")
        }
    }

    #[test]
    fn test_interpret_foreach_effect() {
        let trigger_txt = SAPText::new("Start of turn");
        let effect_txt =
            SAPText::new("Gain +1 attack and +1 health until end of battle for each gold over 10.");

        for token in effect_txt.tokenize().unwrap().iter() {
            println!("{token}")
        }
    }

    #[test]
    fn test_interpret_summon_effect() {
        let trigger_txt = SAPText::new("Enemy summoned");
        let effect_txt = SAPText::new("Summon one Loyal Chinchilla.");

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

    #[test]
    fn test_interpret_max_pet_effect() {
        let trigger_txt = SAPText::new("Enemy summoned");
        let effect_txt =
            SAPText::new("Deal 100% attack damage to the least healthy enemy and itself.");
        let effect_tokens = effect_txt.tokenize().unwrap();

        let triggers: Vec<EffectTrigger> = trigger_txt.tokenize().unwrap().try_into().unwrap();

        // for token in effect_tokens.iter() {
        //     println!("{token}")
        // }
        for trigger in triggers {
            let effect = Effect::new(trigger, &effect_tokens).unwrap();
            println!("{effect:?}")
        }
    }
}
