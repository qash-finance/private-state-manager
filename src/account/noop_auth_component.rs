use std::sync::LazyLock;

use miden_objects::{account::AccountComponent, assembly::{Assembler, Library}};

const NO_AUTH_CODE: &str = "
export.auth_no_auth
    push.0 drop
end";

static NO_AUTH_LIBRARY: LazyLock<Library> = LazyLock::new(|| {
    Assembler::default()
        .assemble_library([NO_AUTH_CODE])
        .expect("noop auth code should be valid")
});

pub struct NoAuthComponent;

impl From<NoAuthComponent> for AccountComponent {
    fn from(_: NoAuthComponent) -> Self {
        AccountComponent::new(NO_AUTH_LIBRARY.clone(), vec![])
            .expect("component should be valid")
            .with_supports_all_types()
    }
}