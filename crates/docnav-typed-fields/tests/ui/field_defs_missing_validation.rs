use docnav_typed_fields::{ProcessStrategy, FieldDef, FieldDefs};

#[derive(FieldDefs)]
struct Params {
    #[field(
        FieldDef::builder("docnav.defaults.limit_chars")
            .process("config", ProcessStrategy::json_path(["defaults", "limit_chars"]))
    )]
    limit_chars: i64,
}

fn main() {}
