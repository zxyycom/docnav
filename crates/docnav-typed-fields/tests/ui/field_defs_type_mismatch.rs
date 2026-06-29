use docnav_typed_fields::{ProcessStrategy, FieldDef, FieldDefs, FieldValidation};

#[derive(FieldDefs)]
struct Params {
    #[field(
        FieldDef::builder("docnav.defaults.limit")
            .process("config", ProcessStrategy::json_path(["defaults", "limit"]))
            .validation(FieldValidation::string())
    )]
    limit: i64,
}

fn main() {}
