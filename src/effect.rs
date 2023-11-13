use std::iter::Peekable;

use anyhow::{bail, Context};

use crate::{
    token::{
        actions::ActionType, attribute::EntityType, logic::LogicType, numeric::NumericType,
        position::PositionType, target::TargetType, types::TokenType, SAPTokens, Token,
    },
    trigger::EffectTrigger,
};

/// A Super Auto Pets effect.
/// - ex. `Gain +2 attack and +2 health.`
#[derive(Debug, Default, PartialEq)]
pub struct Effect<'src> {
    /// Effect trigger.
    pub trigger: Option<EffectTrigger<'src>>,
    /// Secondary effect trigger for conditional effects.
    pub cond_trigger: Option<EffectTrigger<'src>>,
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

/// Macro to update `effect` if the effect is related to the maximum or minimum attack/health of something.
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

/// Macro to advance a peekable iterable returning the result of conditional checks on elements.
///
/// ### Params
/// * `iter` - an iterable.
/// * `cond` - closures taking an element of `iter` and returning a `bool`.
///
/// ### Returns
/// * Last matching element in chain.
#[macro_export]
macro_rules! matches_peek_next {
    // Base case.
    ($iter:ident, $cond:expr) => {
        $iter.next_if($cond)
    };
    // Call continuously.
    ($iter:ident, $cond:expr, $($conds:expr),+) => {
        $iter.next_if($cond).and_then(|_| matches_peek_next!($iter, $($conds), +))
    };
}

/// Update effect trigger from tokens.
macro_rules! update_effect_trigger_from_token {
    ($tokens:ident, $token:ident, $effect_trigger:ident) => {
        match &$token.ttype {
            TokenType::Numeric(NumericType::Number(Some(num))) => {
                $effect_trigger.number = usize::try_from(*num).ok()
            }
            TokenType::Entity(entity) => $effect_trigger.entity = Some(entity.clone()),
            TokenType::Position(pos) => {
                if $effect_trigger.prim_pos.is_none() {
                    $effect_trigger.prim_pos = Some(*pos)
                } else if $effect_trigger.sec_pos.is_none() {
                    $effect_trigger.sec_pos = Some(*pos)
                }
            }
            TokenType::Target(target) => $effect_trigger.target = Some(*target),
            TokenType::Action(action) => $effect_trigger.action = Some(*action),
            TokenType::Logic(logic) => {
                $effect_trigger.logic = Some(*logic);

                // Check for specifically start of battle since made of multple tokens.
                if matches_peek_next!(
                    $tokens,
                    |token| token.ttype == TokenType::Logic(LogicType::Start),
                    |token| token.ttype == TokenType::Entity(EntityType::Battle(None)),
                    |token| token.ttype == TokenType::Entity(EntityType::Ability(None))
                )
                .is_some()
                {
                    $effect_trigger.entity = Some(EntityType::Ability(Some("Start of battle")));
                }
            }
            _ => {}
        }
    };
}

/// Create [`EffectTrigger`] for a [`LogicType::If`] effect.
/// * This should be invoked **before** the current [`Token`] has a [`Token::ttype`] of [`LogicType::ForEach`].
/// * Consumes iterator until [`TokenType::Action`] is found.
///
/// ### Params
/// * `tokens`: [`Peekable`] iterator of tokens.
///
/// ### Returns
/// * [`EffectTrigger`]
fn create_if_cond<'src, T>(tokens: &mut Peekable<T>) -> Option<EffectTrigger<'src>>
where
    T: Iterator<Item = &'src Token<'src>>,
{
    tokens.next_if(|token| matches!(token.ttype, TokenType::Logic(LogicType::If)))?;

    let mut effect_trigger = EffectTrigger {
        logic: Some(LogicType::If),
        ..Default::default()
    };
    while let Some(token) = tokens.next_if(|token| !matches!(token.ttype, TokenType::Action(_))) {
        update_effect_trigger_from_token!(tokens, token, effect_trigger);
    }
    Some(effect_trigger)
}

/// Create [`EffectTrigger`] for a [`LogicType::ForEach`] effect.
/// * This should be invoked when the current [`Token`] has a [`Token::ttype`] of [`LogicType::ForEach`].
/// * Consumes iterator until [`TokenType::EndText`] or [`TokenType::Logic(LogicType::To)`] are found.
///
/// ### Params
/// * `tokens`: [`Peekable`] iterator of tokens.
///
/// ### Returns
/// * [`EffectTrigger`]
fn create_foreach_cond<'src, T>(tokens: &mut Peekable<T>) -> EffectTrigger<'src>
where
    T: Iterator<Item = &'src Token<'src>>,
{
    let mut effect_trigger = EffectTrigger {
        logic: Some(LogicType::ForEach),
        ..Default::default()
    };

    // For each effects consume tokens until LogicType::To or end of text.
    while let Some(token) = tokens.next_if(|token| {
        !matches!(
            token.ttype,
            TokenType::EndText | TokenType::Logic(LogicType::To)
        )
    }) {
        update_effect_trigger_from_token!(tokens, token, effect_trigger);
    }
    effect_trigger
}
impl<'src> Effect<'src> {
    /// Initialize a new SAP effect.
    ///
    /// ### Params
    /// * `trigger`
    /// * `tokens`
    fn new(trigger: Option<EffectTrigger>, tokens: &'src SAPTokens) -> anyhow::Result<Vec<Self>> {
        let mut tokens = tokens.iter().peekable();
        let mut effects: Vec<Effect> = vec![];
        let mut effect = Effect {
            // Construct secondary trigger for If, if possible.
            cond_trigger: create_if_cond(&mut tokens),
            ..Default::default()
        };

        while let Some(token) = tokens.next() {
            match &token.ttype {
                TokenType::Numeric(NumericType::Max) => {
                    update_effect_max_min_stat_pos!(
                        tokens,
                        effect,
                        atk = PositionType::Strongest,
                        health = PositionType::Healthiest
                    );
                }
                TokenType::Numeric(NumericType::Min) => {
                    update_effect_max_min_stat_pos!(
                        tokens,
                        effect,
                        atk = PositionType::Weakest,
                        health = PositionType::Illest
                    );
                }
                TokenType::Numeric(num) => {}
                TokenType::Entity(entity) => {
                    // Consume next token if damage attribute.
                    // This is attack/attack perc damage.
                    if matches!(entity, EntityType::Attack(_) | EntityType::AttackPercent(_)) {
                        tokens.next_if(|token| {
                            matches!(token.ttype, TokenType::Entity(EntityType::Damage(None)))
                        });
                    }
                    effect.entities.push(entity.clone())
                }
                TokenType::EndText => {}
                TokenType::Position(pos) => effect.position.push(*pos),
                TokenType::Target(target) => effect.target = Some(*target),
                // Create new effect trigger for for each effects.
                // We cannot create multiple effects since we won't know stats/attributes of pets until runtime.
                TokenType::Logic(LogicType::ForEach) => {
                    effect.cond_trigger = Some(create_foreach_cond(&mut tokens));
                }
                // Temp effect.
                TokenType::Logic(LogicType::Until) => {
                    // Must be until, end, and battle(none)
                    effect.temp = matches_peek_next!(
                        tokens,
                        |token| token.ttype == TokenType::Logic(LogicType::End),
                        |token| token.ttype == TokenType::Entity(EntityType::Battle(None))
                    )
                    .is_some();
                }
                // Multi-effect
                TokenType::Logic(LogicType::And | LogicType::Or) => {
                    // If next token is action, create new effect.
                    if let Some(TokenType::Action(_)) = tokens.peek().map(|token| &token.ttype) {
                        let mut new_effect = Effect::default();
                        std::mem::swap(&mut effect, &mut new_effect);

                        new_effect.validate_action()?;
                        effects.push(new_effect)
                    }
                }
                TokenType::Logic(LogicType::Works) => {
                    let next_usage_token = matches_peek_next!(tokens, |token| matches!(
                        token.ttype,
                        TokenType::Numeric(NumericType::Multiplier(_))
                    ));
                    if let Some(TokenType::Numeric(NumericType::Multiplier(Some(num_uses)))) =
                        next_usage_token.map(|token| &token.ttype)
                    {
                        // Consume turns token stopping if not present.
                        tokens
                            .next_if(|token| {
                                token.ttype == TokenType::Entity(EntityType::Turn(None))
                            })
                            .context("Must have Turns token after number of uses.")?;
                        // Set number of uses.
                        effect.uses = Some(usize::try_from(*num_uses)?)
                    }
                }
                TokenType::Logic(logic) => {}
                TokenType::Action(action) => effect.action = Some(*action),
            }
        }

        effect.validate_action()?;
        effects.push(effect);
        Ok(effects)
    }

    /// Validate action
    /// * [ActionType::Gain] should only be used on self.
    /// * [ActionType::Give] can be used on other pets.
    fn validate_action(&mut self) -> anyhow::Result<()> {
        match self.action {
            Some(ActionType::Gain) => {
                // Add implicit position if none given.
                let is_trumpet_effect = self
                    .entities
                    .iter()
                    .any(|e| matches!(e, EntityType::Trumpet(_) | EntityType::TrumpetPercent(_)));
                if self.position.is_empty() && !is_trumpet_effect {
                    self.position.push(PositionType::OnSelf)
                }
                // Gain can only affect up to 1 pet.
                if self.position.len() > 1 {
                    bail!("Only one pet can be affected by {:?}", self.action)
                }
                // Gain effect can only affect self.
                if self
                    .position
                    .first()
                    .filter(|pos| **pos != PositionType::OnSelf)
                    .is_some()
                    && !is_trumpet_effect
                {
                    bail!("Gain action only affects Self pet.")
                }
            }
            Some(ActionType::Give) => {
                // Give must always have a position.
                if self.position.is_empty() {
                    bail!("Position must be given for {:?}", self.action)
                }
            }
            Some(_) | None => {}
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{
        token::{
            actions::ActionType, attribute::EntityType, logic::LogicType, position::PositionType,
            target::TargetType,
        },
        trigger::EffectTrigger,
        SAPText,
    };

    use super::Effect;

    #[test]
    fn test_interpret_conditional_has_effect() {
        let effect_txt =
            SAPText::new("If this has a level 3 friend, gain +1 attack and +2 health.");

        let tokens = effect_txt.tokenize().unwrap();
        let effects = Effect::new(None, &tokens).unwrap();

        assert_eq!(effects.len(), 1);
        assert_eq!(
            effects[0],
            Effect {
                trigger: None,
                cond_trigger: Some(EffectTrigger {
                    entity: Some(EntityType::Level(Some(3))),
                    target: Some(TargetType::Friend),
                    logic: Some(LogicType::Have),
                    ..Default::default()
                }),
                target: None,
                entities: vec![EntityType::Attack(Some(1)), EntityType::Health(Some(2))],
                position: vec![PositionType::OnSelf],
                action: Some(ActionType::Gain),
                uses: None,
                temp: false
            }
        )
    }

    #[test]
    fn test_interpret_conditional_is_effect() {
        let effect_txt =
            SAPText::new("If this is your highest tier friend, gain +1 attack and +2 health.");

        let tokens = effect_txt.tokenize().unwrap();
        let effects = Effect::new(None, &tokens).unwrap();
        assert_eq!(effects.len(), 1);
        assert_eq!(
            effects[0],
            Effect {
                trigger: None,
                cond_trigger: Some(EffectTrigger {
                    action: None,
                    number: None,
                    target: Some(TargetType::Friend),
                    prim_pos: Some(PositionType::OnSelf),
                    logic: Some(LogicType::Is),
                    sec_pos: Some(PositionType::Highest),
                    entity: Some(EntityType::Tier(None)),
                }),
                target: None,
                entities: vec![EntityType::Attack(Some(1)), EntityType::Health(Some(2))],
                position: vec![PositionType::OnSelf],
                action: Some(ActionType::Gain),
                uses: None,
                temp: false
            }
        )
    }

    #[test]
    fn test_interpret_conditional_battle_effect() {
        let effect_txt = SAPText::new("If in battle, gain +1 attack and +2 health.");
        let tokens = effect_txt.tokenize().unwrap();
        let effects = Effect::new(None, &tokens).unwrap();

        assert_eq!(effects.len(), 1);
        assert_eq!(
            effects[0],
            Effect {
                trigger: None,
                cond_trigger: Some(EffectTrigger {
                    action: None,
                    number: None,
                    entity: Some(EntityType::Battle(None)),
                    target: None,
                    logic: Some(LogicType::In),
                    prim_pos: None,
                    sec_pos: None
                }),
                target: None,
                entities: vec![EntityType::Attack(Some(1)), EntityType::Health(Some(2))],
                position: vec![PositionType::OnSelf],
                action: Some(ActionType::Gain),
                uses: None,
                temp: false
            }
        )
    }

    #[test]
    fn test_interpret_conditional_toy_effect() {
        let effect_txt =
            SAPText::new("If you have a toy, give the nearest friend behind +10 health.");

        let tokens = effect_txt.tokenize().unwrap();
        let effects = Effect::new(None, &tokens).unwrap();

        assert_eq!(effects.len(), 1);
        assert_eq!(
            effects[0],
            Effect {
                trigger: None,
                cond_trigger: Some(EffectTrigger {
                    action: None,
                    number: None,
                    entity: Some(EntityType::Toy(None)),
                    target: None,
                    logic: Some(LogicType::Have),
                    prim_pos: None,
                    sec_pos: None,
                },),
                target: Some(TargetType::Friend),
                entities: vec![EntityType::Health(Some(10))],
                position: vec![PositionType::Nearest, PositionType::Behind],
                action: Some(ActionType::Give),
                uses: None,
                temp: false,
            }
        )
    }

    #[test]
    fn test_interpret_conditional_start_battle_effect() {
        let effect_txt = SAPText::new("If it has a Start of battle ability, gain +2 attack.");
        let tokens = effect_txt.tokenize().unwrap();
        let effects = Effect::new(None, &tokens).unwrap();

        assert_eq!(
            effects[0],
            Effect {
                trigger: None,
                cond_trigger: Some(EffectTrigger {
                    action: None,
                    number: None,
                    entity: Some(EntityType::Ability(Some("Start of battle"))),
                    target: None,
                    logic: Some(LogicType::Have),
                    prim_pos: Some(PositionType::Trigger),
                    sec_pos: None
                }),
                target: None,
                entities: vec![EntityType::Attack(Some(2))],
                position: vec![PositionType::OnSelf],
                action: Some(ActionType::Gain),
                uses: None,
                temp: false
            }
        )
    }

    #[test]
    fn test_interpret_conditional_invalid_multi_use_effect() {
        let invalid_effect_txt = SAPText::new(
            "If it was a Faint pet, activate its ability again. Works 1 time per game.",
        );
        let invalid_tokens = invalid_effect_txt.tokenize().unwrap();
        // Works per turn only.
        assert!(Effect::new(None, &invalid_tokens).is_err());
    }

    #[test]
    fn test_interpret_conditional_multi_use_effect() {
        let effect_txt = SAPText::new(
            "If it was a Faint pet, activate its ability again. Works 1 time per turn.",
        );

        let tokens = effect_txt.tokenize().unwrap();
        let effects = Effect::new(None, &tokens).unwrap();

        assert_eq!(
            effects[0],
            Effect {
                trigger: None,
                cond_trigger: Some(EffectTrigger {
                    action: None,
                    number: None,
                    entity: Some(EntityType::Pet {
                        number: None,
                        name: None,
                        attr: Some("Faint")
                    }),
                    target: None,
                    logic: Some(LogicType::If),
                    prim_pos: Some(PositionType::Trigger),
                    sec_pos: None
                }),
                target: None,
                entities: vec![EntityType::Ability(None)],
                position: vec![PositionType::Trigger],
                action: Some(ActionType::Activate),
                uses: Some(1),
                temp: false
            }
        )
    }

    #[test]
    fn test_interpret_foreach_effect() {
        let trigger_txt = SAPText::new("Start of turn");
        let effect_txt =
            SAPText::new("Gain +1 attack and +1 health until end of battle for each gold over 10.");
        let effect_middle_txt =
            SAPText::new("Deal 2 damage for each Strawberry friend to one random enemy.");

        let effect_tokens = effect_middle_txt.tokenize().unwrap();

        for token in effect_middle_txt.tokenize().unwrap().iter() {
            println!("{token}")
        }

        let effect_triggers =
            TryInto::<Vec<EffectTrigger>>::try_into(trigger_txt.tokenize().unwrap()).unwrap();
        for trigger in effect_triggers {
            let effect = Effect::new(Some(trigger), &effect_tokens).unwrap();
            println!("{effect:?}")
        }
        todo!()
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
        todo!()
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
            let effect = Effect::new(Some(trigger), &effect_tokens).unwrap();
            println!("{effect:?}")
        }
        todo!()
    }
}
