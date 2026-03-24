use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CompilerConfig {
    pub debug: bool,
    pub features: FeatureFlags,
}

#[derive(Debug, Deserialize)]
pub struct FeatureFlags {
    pub class_casting: bool,
    pub io: bool,
    pub string: bool,
    pub super_kw: bool,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            debug: false,
            features: FeatureFlags { class_casting: true, string: true, io: false, super_kw: true },
        }
    }
}

#[derive(Debug, Clone)]
pub enum ShipFeature {
    ClassCasting,
    Io,
    String,
    SuperKeyword,
}
impl ShipFeature {
    pub fn name(&self) -> &'static str {
        match self {
            ShipFeature::ClassCasting => "class_casting",
            ShipFeature::Io => "io",
            ShipFeature::String => "string",
            ShipFeature::SuperKeyword => "super_kw",
        }
    }

    pub fn is_enabled(&self, features: &FeatureFlags) -> bool {
        match self {
            ShipFeature::ClassCasting => features.class_casting,
            ShipFeature::Io => features.io,
            ShipFeature::String => features.string,
            ShipFeature::SuperKeyword => features.super_kw,
        }
    }
}
