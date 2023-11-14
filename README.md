# Super Auto Lex
Lexer to parse SAP Pet effect text into a stream of tokens.

### Usage
```bash
cargo add --git https://github.com/koisland/SuperAutoLex
```

Create an effect without an effect trigger.
```rust
use saplex::{SAPText, Effect};

// Tokenize some SAP related text.
let effect_txt = SAPText::new("If this has a level 3 friend, gain +1 attack and +2 health.");
let tokens = effect_txt.tokenize().unwrap();

// Pass effect tokens and optionally, an effect trigger, to generate an effect.
let effects: Vec<Effect> = Effect::new(None, &tokens).unwrap();
```

Create an effect trigger.
```rust
use saplex::{SAPText, EffectTrigger};

let trigger_txt = SAPText::new("End turn & Start of battle");
let trigger_tokens = trigger_txt.tokenize().unwrap();
let effect_trigger: Vec<EffectTrigger> = trigger_tokens.try_into().unwrap();
```

Create an effect.
```rust
use saplex::{SAPText, EffectTrigger, Effect};

// Define effect text.
let trigger_txt = SAPText::new("Enemy summoned");
let effect_txt =
    SAPText::new("Deal 100% attack damage to the least healthy enemy and itself.");

// Create tokens.
let effect_tokens = effect_txt.tokenize().unwrap();
let trigger_tokens = trigger_txt.tokenize().unwrap();

// Create effect trigger.
let effect_trigger = {
    let mut effect_trigger: Vec<EffectTrigger> = trigger_tokens.try_into().unwrap();
    effect_trigger.remove(0)
};

// And finally, create the effect.
let effect = Effect::new(Some(effect_trigger), &effect_tokens).unwrap();
```

Enable the `serde` feature flag to serialize and deserialize effects.
```bash
cargo add --git https://github.com/koisland/SuperAutoLex --features serde
```

To serialize and deserialize effects:
```rust
use saplex::{SAPText, EffectTrigger, Effect};

let effect_txt = SAPText::new("If in battle, gain +1 attack and +2 health.");
let tokens = effect_txt.tokenize().unwrap();
let effects = Effect::new(None, &tokens).unwrap();

let effect_txt_json: String = serde_json::to_string(&effects[0]).unwrap();
let effect: Effect = serde_json::from_str(&effect_txt_json).unwrap();
```

### Rules
Item names are always uppercase.
* Pets can be one or two words long.
    * `Dog`
    * `Lizard Tail`
* Foods end with `Perk`.
    * `Melon Perk`
    * Can also omitted if prior word is `with`.
        * `Dog with Melon.`
* WIP

### TODO
* [ ] Declarative macro to construct effects.
