use pgrx::prelude::*;
use oso::{Oso, PolarClass};
use serde::Deserialize;
use uuid::Uuid;
use log::debug;

pgrx::pg_module_magic!();

const OSO_SCHEMA: &str = "
actor AclUser {}

resource AclResources {
  permissions = [\"list\", \"details\", \"create\", \"accept\", \"update\", \"delete\"];
  roles = [\"readonly\", \"manager\"];

  \"readonly\" if \"manager\";

  \"list\" if \"readonly\";
  \"details\" if \"readonly\";

  \"create\" if \"manager\";
  \"accept\" if \"manager\";
  \"update\" if \"manager\";
  \"delete\" if \"manager\";
}

has_role(actor: AclUser, role_name: String, _: AclResources) if
  role_name = actor.role;

allow(actor: AclUser, action, resource) if
  has_permission(actor, action, resource);

";

#[derive(PolarClass, Clone)]
enum AclResources {
    Memberships,
    Invitations,
    Organizations,
    Operators,
    Users,
}

#[derive(Deserialize, PolarClass, Clone, Debug)]
pub struct AclUser {
    pub id: Uuid,

    #[polar(attribute)]
    pub role: String,
}

#[pg_extern]
fn hello_pgzan() -> &'static str {
    "Hello, pgzan"
}

#[pg_extern]
fn pgzan_check(requestor: String) -> bool {
    let req: AclUser = serde_json::from_str(&requestor).expect("failed to deserialize");

    // Load the oso evaluator
    let mut oso = Oso::new();
    oso.register_class(AclResources::get_polar_class()).expect("failed to register AclResources");
    oso.register_class(AclUser::get_polar_class()).expect("failed to register AclUser");
    if let Err(err) = oso.load_str(&OSO_SCHEMA) {
        error!("failed to load polar schema\ngot error: {:#?}", err);
    }

    // Now try to evaluate the permissions
    match oso.is_allowed(req, "update", AclResources::Organizations)
    {
        Ok(true) => {
            debug!("Authorized");
            true
        },
        Ok(false) => {
            debug!("Unauthorized: operator is not authorized");
            false
        }
        Err(err) => {
            debug!("Unauthorized: failure during authorization: {:?}", err);
            false
        }
    }
}



#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::prelude::*;

    #[pg_test]
    fn test_hello_pgzan() {
        assert_eq!("Hello, pgzan", crate::hello_pgzan());
    }

}

/// This module is required by `cargo pgrx test` invocations.
/// It must be visible at the root of your extension crate.
#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}
