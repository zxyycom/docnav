use docnav_typed_fields::{ExtractStrategy, FieldDef, FieldDefs, FieldValidation};

#[derive(FieldDefs)]
struct Params {
    #[field(
        FieldDef::builder("docnav.defaults.limit_chars")
            .extract("config", ExtractStrategy::json_path(["defaults", "limit_chars"]))
            .validation(FieldValidation::string())
    )]
    limit_chars: i64,
}

fn main() {}
