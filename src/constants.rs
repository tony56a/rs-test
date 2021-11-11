use rusoto_core::Region;

pub const DEEPAI_TOKEN_KEY: &str = "deepai_token";
// The build hash (loaded from (Circle)CI builds)
pub const BUILD_HASH_KEY: &str = "build_hash";
// THe Git project link template
pub const GIT_REPO_LINK_TEMPLATE: &str = "https://github.com/tony56a/rs-test/commit/";
// Non CI indicator string
pub const DEVELOPMENT_BUILD: &str = "development";
// File path to secret menu DB file
pub const SECRET_MENU_DB_PATH: &str = "resources/db/secrets.db";

pub const AWS_RESOURCE_REGION: Region = Region::UsEast1;
