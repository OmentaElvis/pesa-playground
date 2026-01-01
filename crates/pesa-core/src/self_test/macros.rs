/// Defines the `SelfTests` enum and the `all_tests` function, which together
/// constitute the full, ordered suite of self-tests.
///
/// This macro is the central point of registration for all `TestStep` implementations.
/// The order of declaration in the macro determines the order of execution.
///
/// # Syntax
///
/// The macro takes a comma-separated list of test definitions. Each definition
/// has a `VariantName` for the enum, a UI display `name`, a `description`, and
/// a `ctor` (constructor) which should be an expression that creates an instance
/// of a struct that implements the `TestStep` trait.
///
/// ```rust,ignore
/// define_tests!(
///     // The enum variant for this test
///     TestVariantName {
///         // The string name displayed in the UI
///         name: "my_test",
///         // The description displayed in the UI
///         description: "Tests a specific feature.",
///         // An expression that constructs the TestStep struct
///         ctor: path::to::MyTestStepStruct
///     },
///     // ... more tests
/// );
/// ```
///
/// # Example
///
/// ```rust
/// // in `my_test.rs`
/// // pub struct MyTestStep;
/// // impl TestStep for MyTestStep { ... }
///
/// // in `mod.rs`
/// mod my_test;
///
/// define_tests!(
///     CreateProject {
///         name: "create_project",
///         description: "Tests project creation",
///         ctor: crate::self_test::tests::create_project::CreateProjectTest
///     },
///     MyTest {
///         name: "my_test",
///         description: "This is a new test.",
///         ctor: my_test::MyTestStep
///     }
/// );
/// ```
#[macro_export]
macro_rules! define_tests {
    (
        $(
            $variant:ident {
                name: $name:expr,
                description: $desc:expr,
                ctor: $ctor:expr $(,)?
            }
        ),+ $(,)?
    ) => {
        pub enum SelfTests {
            $(
                $variant,
            )+
        }

        impl SelfTests {
            pub fn name(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant => $name,
                    )+
                }
            }

            pub fn description(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant => $desc,
                    )+
                }
            }

            pub async fn run(
                &self,
                ctx: &mut $crate::self_test::context::TestContext,
                cb: &mut $crate::self_test::callback::CallbackManager,
            ) -> anyhow::Result<()> {
                match self {
                    $(
                        Self::$variant => {
                            let t = $ctor;
                            t.run(ctx, cb).await
                        }
                    )+
                }
            }
        }

        pub fn all_tests() -> Vec<SelfTests> {
            vec![
                $(
                    SelfTests::$variant,
                )+
            ]
        }
    };
}
