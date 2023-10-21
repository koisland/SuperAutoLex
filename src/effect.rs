use anyhow::bail;

use crate::token::{
    actions::ActionType, attribute::EntityType, logic::LogicType, numeric::NumericType,
    position::PositionType, target::TargetType, types::TokenType, SAPTokens,
};

#[derive(Debug, Default, PartialEq)]
pub struct EffectTrigger {
    /// Action
    pub action: Option<ActionType>,
    /// Number of trigger, if any.
    pub number: Option<usize>,
    /// Entity type.
    pub entity: Option<EntityType>,
    /// The target type.
    pub target: Option<TargetType>,
    /// Logic type.
    pub logic: Option<LogicType>,
    /// General position on `affected_team`.
    pub position: Option<PositionType>,
}

impl<'src> TryFrom<SAPTokens<'src>> for Vec<EffectTrigger> {
    type Error = anyhow::Error;

    fn try_from(tokens: SAPTokens) -> Result<Self, Self::Error> {
        let mut trigger = EffectTrigger::default();
        let mut triggers = vec![];
        let mut tokens = tokens.iter().peekable();

        // TODO: May need to lookahead at token ahead to figure out.
        while let Some(token) = tokens.next() {
            match token.ttype {
                TokenType::Numeric(NumericType::Number(Some(num))) => {
                    trigger.number = Some(num.try_into()?)
                }
                TokenType::Entity(entity) => {
                    trigger.entity = Some(entity);
                    trigger.number = entity.value().and_then(|val| usize::try_from(val).ok());
                }
                TokenType::Position(pos) => trigger.position = Some(pos),
                TokenType::Target(target) => trigger.target = Some(target),
                TokenType::Action(action) => {
                    if action.is_shop_related() {
                        trigger.target = Some(TargetType::Shop);
                    }
                    trigger.action = Some(action)
                }
                TokenType::Logic(LogicType::And | LogicType::Or) => {
                    // Look at next token.
                    let next_token_type = tokens.peek().map(|token| &token.ttype);
                    match next_token_type {
                        // If new action next or another thing...
                        Some(TokenType::Action(_)) | Some(_) => {
                            // Create new trigger with same action.
                            // Swap with old one.
                            // And push old into triggers.
                            let mut new_trigger = EffectTrigger {
                                action: trigger.action,
                                ..Default::default()
                            };

                            std::mem::swap(&mut new_trigger, &mut trigger);
                            triggers.push(new_trigger);
                        }
                        None => {
                            bail!(
                                "Syntax error. Logical statement ({:?}) without associated value.",
                                token.ttype
                            )
                        }
                    }
                }
                TokenType::Logic(logic) => trigger.logic = Some(logic),
                _ => {}
            }
        }

        // Add workhorse trigger.
        triggers.push(trigger);
        Ok(triggers)
    }
}

pub struct Effect {
    /// Effect trigger.
    pub trigger: Option<EffectTrigger>,
    /// Target of the effect.
    pub target: Option<TargetType>,
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

impl Effect {
    fn new(trigger: EffectTrigger, tokens: SAPTokens) {}
}

#[cfg(test)]
mod tests {
    use crate::{
        token::{actions::ActionType, attribute::EntityType, logic::LogicType, target::TargetType},
        SAPText,
    };

    use super::EffectTrigger;

    #[test]
    fn test_interpret_positional_effect_trigger() {
        let triggers: Vec<EffectTrigger> = SAPText::new("Two friends ahead faint")
            .tokenize()
            .unwrap()
            .try_into()
            .unwrap();

        println!("{triggers:?}")
    }
    #[test]
    fn test_interpret_numeric_effect_trigger() {
        let triggers: Vec<EffectTrigger> = SAPText::new("Two friends faint")
            .tokenize()
            .unwrap()
            .try_into()
            .unwrap();

        assert_eq!(
            triggers,
            [EffectTrigger {
                action: Some(ActionType::Faint),
                number: Some(2),
                entity: None,
                target: Some(TargetType::Friend),
                logic: None,
                position: None
            }]
        )
    }

    #[test]
    fn test_interpret_multiple_effect_trigger_w_one_action() {
        let triggers: Vec<EffectTrigger> = SAPText::new("Gain perk or ailment")
            .tokenize()
            .unwrap()
            .try_into()
            .unwrap();
        let verbose_triggers: Vec<EffectTrigger> = SAPText::new("Gain perk or gain ailment")
            .tokenize()
            .unwrap()
            .try_into()
            .unwrap();
        let exp_triggers = vec![
            EffectTrigger {
                action: Some(ActionType::Gain),
                number: None,
                entity: Some(EntityType::Perk(None)),
                target: None,
                logic: None,
                position: None,
            },
            EffectTrigger {
                action: Some(ActionType::Gain),
                number: None,
                entity: Some(EntityType::Ailment(None)),
                target: None,
                logic: None,
                position: None,
            },
        ];
        assert_eq!(triggers, exp_triggers);
        assert_eq!(verbose_triggers, exp_triggers)
    }

    #[test]
    fn test_interpret_effect_trigger() {
        let triggers: Vec<EffectTrigger> = SAPText::new("After attack")
            .tokenize()
            .unwrap()
            .try_into()
            .unwrap();

        assert_eq!(
            triggers,
            vec![EffectTrigger {
                action: None,
                number: None,
                entity: Some(EntityType::Attack(None)),
                target: None,
                logic: Some(LogicType::After),
                position: None,
            }]
        )
    }

    #[test]
    fn test_interpret_multiple_effect_trigger() {
        let triggers: Vec<EffectTrigger> = SAPText::new("After attack or before attack")
            .tokenize()
            .unwrap()
            .try_into()
            .unwrap();

        assert_eq!(
            triggers,
            vec![
                EffectTrigger {
                    action: None,
                    number: None,
                    entity: Some(EntityType::Attack(None)),
                    target: None,
                    logic: Some(LogicType::After),
                    position: None,
                },
                EffectTrigger {
                    action: None,
                    number: None,
                    entity: Some(EntityType::Attack(None)),
                    target: None,
                    logic: Some(LogicType::Before),
                    position: None,
                }
            ]
        )
    }
}
