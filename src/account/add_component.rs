use std::sync::LazyLock;

use miden_objects::{account::AccountComponent, assembly::{Assembler, Library}};

const ADD_CODE: &str = "
    export.add5
        add.5
    end
";

static ADD_LIBRARY: LazyLock<Library> = LazyLock::new(|| {
    Assembler::default()
        .assemble_library([ADD_CODE])
        .expect("add code should be valid")
});

pub struct AddComponent;

impl From<AddComponent> for AccountComponent {
    fn from(_: AddComponent) -> Self {
        AccountComponent::new(ADD_LIBRARY.clone(), vec![])
            .expect("component should be valid")
            .with_supports_all_types()
    }
}