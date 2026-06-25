use docnav_typed_fields::{FieldDef, FieldDefs};

#[derive(FieldDefs)]
struct Params {
    #[field(FieldDef::builder("docnav.defaults.limit_chars").path(["defaults", "limit_chars"]))]
    limit_chars: i64,
}

fn main() {}
