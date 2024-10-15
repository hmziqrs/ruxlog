diesel_ext -m -d "Queryable, Identifiable, Selectable, Debug, PartialEq" > src/db/models_blueprint.rs

diesel migration run

diesel migration revert -a


diesel migration generate --diff-schema email_verification
