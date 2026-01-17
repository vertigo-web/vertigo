use darling::{FromAttributes, FromMeta};
use heck::{ToKebabCase, ToLowerCamelCase, ToPascalCase, ToShoutySnakeCase, ToSnakeCase};

#[derive(Debug, Clone, Copy, FromMeta)]
pub enum RenameAll {
    #[darling(rename = "PascalCase")]
    PascalCase,
    #[darling(rename = "camelCase")]
    CamelCase,
    #[darling(rename = "snake_case")]
    SnakeCase,
    #[darling(rename = "kebab-case")]
    KebabCase,
    #[darling(rename = "SHOUTY_SNAKE_CASE")]
    ShoutySnakeCase,
    #[darling(rename = "UPPERCASE")]
    UpperCase,
    #[darling(rename = "lowercase")]
    Lowercase,
}

impl RenameAll {
    /// Renames a string according to the case convention.
    pub fn rename(&self, s: &str) -> String {
        match self {
            RenameAll::PascalCase => s.to_pascal_case(),
            RenameAll::CamelCase => s.to_lower_camel_case(), // Serde's "camelCase" is lowerCamelCase
            RenameAll::SnakeCase => s.to_snake_case(),
            RenameAll::KebabCase => s.to_kebab_case(),
            RenameAll::ShoutySnakeCase => s.to_shouty_snake_case(),
            RenameAll::UpperCase => s.to_uppercase(),
            RenameAll::Lowercase => s.to_lowercase(),
        }
    }
}

/// Options for the container (struct or enum)
#[derive(Debug, FromAttributes)]
#[darling(attributes(js_json))]
pub struct ContainerOpts {
    /// Rename all fields according to the given case convention.
    #[darling(default)]
    pub rename_all: Option<RenameAll>,
}

/// Options for a field
#[derive(Default, Debug, FromAttributes)]
#[darling(attributes(js_json), forward_attrs(allow, doc, cfg))]
pub struct FieldOpts {
    /// Default value for the field if it is missing in the JSON.
    pub default: Option<darling::util::Override<syn::Expr>>,
    /// Rename the field to the given string.
    pub rename: Option<String>,
    /// Serialize the field using `Display` and deserialize using `FromStr`.
    #[darling(default)]
    pub stringify: bool,
}
