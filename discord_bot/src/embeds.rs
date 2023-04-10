// use core::unicode::conversions::to_upper;

use proto_rs::character::{
    Ability, Artifact, Cypher, Item, ItemType, Note, NoteType, PoolType, Skill,
    SkillLevel,
};
use serenity::builder::CreateEmbed;

pub trait Embedable {
    fn embed(&self) -> CreateEmbed;
}

impl Embedable for Cypher {
    fn embed(&self) -> CreateEmbed {
        let mut embed = CreateEmbed::default();
        embed.title(format!("[Cypher] {}", self.name));
        if !self.short_description.is_empty() {
            embed.description(self.get_short_description());
        }

        embed.field(
            "Level",
            match self.get_level().is_empty() {
                false => self.get_level(),
                true => "?",
            },
            true,
        );

        if !self.get_depletion().is_empty() {
            embed.field("Depletion", self.get_depletion(), true);
        };

        if !self.get_effect().is_empty() {
            embed.field("Effect", self.get_effect(), false);
        }

        if !self.get_internal().is_empty() {
            embed.field("Internal", self.get_internal(), false);
        }

        if !self.get_wearable().is_empty() {
            embed.field("Wearable", self.get_wearable(), false);
        }

        if !self.get_usable().is_empty() {
            embed.field("Usable", self.get_usable(), false);
        }

        embed
    }
}

impl Embedable for Artifact {
    fn embed(&self) -> CreateEmbed {
        let mut embed = CreateEmbed::default();
        embed.title(format!("[Artifact] {}", self.name));
        if !self.short_description.is_empty() {
            embed.description(self.get_short_description());
        }

        embed.field(
            "Level",
            match self.get_level().is_empty() {
                false => self.get_level(),
                true => "?",
            },
            true,
        );

        if !self.get_depletion().is_empty() {
            embed.field("Depletion", self.get_depletion(), true);
        };

        if !self.get_effect().is_empty() {
            embed.field("Effect", self.get_effect(), false);
        }

        if !self.get_form().is_empty() {
            embed.field("Form", self.get_form(), false);
        }

        embed
    }
}

impl Embedable for Ability {
    fn embed(&self) -> CreateEmbed {
        let mut embed = CreateEmbed::default();
        embed.title(format!("[Ability] {}", self.name));

        let has_short_description = !self.short_description.is_empty();
        let has_description = !self.description.is_empty();

        if has_short_description {
            embed.description(self.get_short_description());
        } else if has_description {
            embed.description(self.get_description());
        }

        embed.field("Type", printable_pool_type(self.get_field_type()), true);

        embed.field(
            "Cost",
            match self.get_cost().is_empty() {
                false => self.get_cost(),
                true => "?",
            },
            true,
        );

        embed.field("Enabler", self.get_enabler(), false);

        if has_description && has_short_description {
            embed.field("Description", self.get_description(), false);
        }

        embed
    }
}

impl Embedable for Skill {
    fn embed(&self) -> CreateEmbed {
        let mut embed = CreateEmbed::default();
        embed.title(format!("[Skill] {}", self.name));

        let has_description = !self.description.is_empty();

        if has_description {
            embed.description(self.get_description());
        }

        embed.field("Type", printable_pool_type(self.get_field_type()), true);

        embed.field("Level", printable_skill_level(self.get_level()), true);

        embed
    }
}

impl Embedable for Item {
    fn embed(&self) -> CreateEmbed {
        let mut embed = CreateEmbed::default();
        embed.title(format!("[Item] {}", self.name));

        let has_short_description = !self.short_description.is_empty();
        let has_description = !self.description.is_empty();

        if has_short_description {
            embed.description(self.get_short_description());
        } else if has_description {
            embed.description(self.get_description());
        }

        let types: Vec<String> =
            self.types.iter().map(|t| printable_item_type(*t)).collect();

        embed.field("Type", types.join(", "), false);

        if self.get_value() > 0.0 {
            embed.field("Value", self.get_value(), true);
        };

        if self.has_armor() && self.get_armor() > 0 {
            embed.field("Armor", self.get_armor(), true);
        }

        if has_description && has_short_description {
            embed.field("Description", self.get_description(), false);
        }

        if self.has_sub_item_type() {
            embed.field(
                "Has Sub-Items",
                printable_item_type(self.get_sub_item_type()),
                false,
            );
        }

        embed
    }
}

impl Embedable for Note {
    fn embed(&self) -> CreateEmbed {
        let mut embed = CreateEmbed::default();
        embed.title(format!("[Note] {}", self.title));

        let has_short_description = !self.shortDescription.is_empty();

        if has_short_description {
            embed.description(self.get_shortDescription());
        }

        embed.field("Type", printable_note_type(self.get_field_type()), true);

        embed.field("Text", self.get_text(), false);

        embed
    }
}

fn printable_item_type(item_type: ItemType) -> String {
    match item_type {
        ItemType::armor => "Armor",
        ItemType::weapon => "Weapon",
        ItemType::clothing => "Clothing",
        ItemType::tool => "Tool",
        ItemType::oddity => "Oddity",
        ItemType::material => "Material",
        ItemType::ammo => "Ammo",
        ItemType::plan => "Plan",
        ItemType::others => "Others",
    }
    .to_owned()
}

fn printable_pool_type(pool_type: PoolType) -> String {
    match pool_type {
        PoolType::intellect => "Intellect",
        PoolType::speed => "Speed",
        PoolType::might => "Might",
    }
    .to_owned()
}

fn printable_skill_level(skill_level: SkillLevel) -> String {
    match skill_level {
        SkillLevel::specialized => "Specialized",
        SkillLevel::trained => "Trained",
        SkillLevel::inability => "Inability",
    }
    .to_owned()
}

fn printable_note_type(note_type: NoteType) -> String {
    match note_type {
        NoteType::misc => "Misc",
        NoteType::location => "Location",
        NoteType::character => "Character",
        NoteType::item => "Item",
        NoteType::quest => "Quest",
    }
    .to_owned()
}
