use anyhow::bail;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::token::{
    actions::ActionType, attribute::EntityType, logic::LogicType, numeric::NumericType,
    position::PositionType, target::TargetType, types::TokenType, SAPTokens,
};

/// A Super Auto Pets effect trigger.
/// - ex. `End turn`
#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EffectTrigger<'src> {
    /// Action
    pub action: Option<ActionType>,
    /// Number of trigger, if any.
    pub number: Option<usize>,
    /// Entity type.
    #[serde(borrow)]
    pub entity: Option<EntityType<'src>>,
    /// The target type.
    pub target: Option<TargetType>,
    /// Logic type.
    pub logic: Option<LogicType>,
    /// Primary position on [`EffectTrigger::target`].
    pub prim_pos: Option<PositionType>,
    /// Secondary position on [`EffectTrigger::target`]. Used in conjunction with [`EffectTrigger::logic`].
    pub sec_pos: Option<PositionType>,
}

impl<'src> TryFrom<SAPTokens<'src>> for Vec<EffectTrigger<'src>> {
    type Error = anyhow::Error;

    fn try_from(tokens: SAPTokens<'src>) -> Result<Self, Self::Error> {
        let mut trigger = EffectTrigger::default();
        let mut triggers = vec![];
        let mut tokens = tokens.iter().peekable();

        while let Some(token) = tokens.next() {
            match token.ttype {
                TokenType::Numeric(NumericType::Number(Some(num))) => {
                    trigger.number = Some(num.try_into()?)
                }
                TokenType::Entity(ref entity) => {
                    trigger.number = entity.value().and_then(|val| usize::try_from(val).ok());
                    trigger.entity = Some(entity.clone());
                }
                TokenType::Position(pos) => trigger.prim_pos = Some(pos),
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

#[cfg(test)]
mod tests {
    use crate::{
        token::{
            actions::ActionType, attribute::EntityType, logic::LogicType, position::PositionType,
            target::TargetType,
        },
        SAPText,
    };
    use pretty_assertions::assert_eq;

    use super::EffectTrigger;

    #[test]
    fn test_interpret_positional_effect_trigger() {
        let txt = SAPText::new("Friend ahead faints");
        let triggers: Vec<EffectTrigger> = txt.tokenize().unwrap().try_into().unwrap();

        assert_eq!(
            triggers,
            [EffectTrigger {
                action: Some(ActionType::Faint),
                target: Some(TargetType::Friend),
                prim_pos: Some(PositionType::Ahead),
                ..Default::default()
            }]
        )
    }
    #[test]
    fn test_interpret_numeric_effect_trigger() {
        let binding = SAPText::new("Two friends faint");
        let triggers: Vec<EffectTrigger> = binding.tokenize().unwrap().try_into().unwrap();

        assert_eq!(
            triggers,
            [EffectTrigger {
                action: Some(ActionType::Faint),
                number: Some(2),
                target: Some(TargetType::Friend),
                ..Default::default()
            }]
        )
    }

    #[test]
    fn test_interpret_multiple_effect_trigger_w_one_action() {
        let txt = SAPText::new("Gain perk or ailment");
        let triggers: Vec<EffectTrigger> = txt.tokenize().unwrap().try_into().unwrap();
        let txt = SAPText::new("Gain perk or gain ailment");
        let verbose_triggers: Vec<EffectTrigger> = txt.tokenize().unwrap().try_into().unwrap();
        let exp_triggers = [
            EffectTrigger {
                action: Some(ActionType::Gain),
                entity: Some(EntityType::Perk(None)),
                ..Default::default()
            },
            EffectTrigger {
                action: Some(ActionType::Gain),
                entity: Some(EntityType::Ailment(None)),
                ..Default::default()
            },
        ];
        assert_eq!(triggers, exp_triggers);
        assert_eq!(verbose_triggers, exp_triggers)
    }

    #[test]
    fn test_interpret_effect_trigger() {
        let txt = SAPText::new("After attack");
        let triggers: Vec<EffectTrigger> = txt.tokenize().unwrap().try_into().unwrap();

        assert_eq!(
            triggers,
            [EffectTrigger {
                entity: Some(EntityType::Attack(None)),
                logic: Some(LogicType::After),
                ..Default::default()
            }]
        )
    }

    #[test]
    fn test_interpret_ampersand_effect_trigger() {
        let sym_txt = SAPText::new("After attack & before attack");
        let txt = SAPText::new("After attack and before attack");

        let sym_triggers: Vec<EffectTrigger> = txt.tokenize().unwrap().try_into().unwrap();
        let triggers: Vec<EffectTrigger> = sym_txt.tokenize().unwrap().try_into().unwrap();

        assert_eq!(sym_triggers, triggers);
    }

    #[test]
    fn test_interpret_multiple_effect_trigger() {
        let txt = SAPText::new("After attack or before attack");
        let triggers: Vec<EffectTrigger> = txt.tokenize().unwrap().try_into().unwrap();

        assert_eq!(
            triggers,
            [
                EffectTrigger {
                    entity: Some(EntityType::Attack(None)),
                    logic: Some(LogicType::After),
                    ..Default::default()
                },
                EffectTrigger {
                    entity: Some(EntityType::Attack(None)),
                    logic: Some(LogicType::Before),
                    ..Default::default()
                }
            ]
        )
    }

    #[test]
    fn test_interpret_battle_turn_trigger() {
        let txt = SAPText::new("Start of battle");
        let triggers: Vec<EffectTrigger> = txt.tokenize().unwrap().try_into().unwrap();

        assert_eq!(
            *triggers,
            [EffectTrigger {
                entity: Some(EntityType::Battle(None)),
                logic: Some(LogicType::Start),
                ..Default::default()
            }]
        )
    }
}
