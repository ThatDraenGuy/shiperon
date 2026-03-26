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

impl CompilerConfig {
    pub fn no_features() -> Self {
        Self {
            debug: false,
            features: FeatureFlags {
                class_casting: false,
                string: false,
                io: false,
                super_kw: false,
            },
        }
    }

    pub fn all_features() -> Self {
        Self {
            debug: false,
            features: FeatureFlags { class_casting: true, string: true, io: true, super_kw: true },
        }
    }

    pub fn with_debug(self) -> Self {
        Self { debug: true, ..self }
    }
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self::all_features()
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
