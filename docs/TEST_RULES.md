# Rules for Testing ("Test Rules")

1. Use `cucumber` for integration tests. Place these tests in the `tests/` directory.
1. The integration tests would act differently on local machine and CI machines on GitHub. Make sure to the tests are all green on both environments.
1. Do not use `cucumber` for unit tests. Use standard Rust unit tests with `#[cfg(test)]` and `#[test]` attributes. Do no use `tests/` directory for unit tests.
1. Use `tracing` for logging in tests. Do not use `log` crate or `println!`/`eprintln` macros. You can control the output of `tracing` using the `BLUELINE_LOG_LEVEL` environment variable.
1. Do not let users to test the code on your behalf. All tests should capture what users can capture on the tests code. Do not ask users to perform manual tests repreatedly.
1. Use `assert` in all `then` steps of `cucumber` tests. `then` without `assert` is not a valid test and must be eliminated.
1. Skipped `then` steps in `cucumber` tests are not acceptable. You have to have all `then` steps implemented with `assert` statements and should be successful.
1. Never use `assert!(true, "message")` to fake the test. If you need to skip a test, use `#[ignore]` attribute on the test function.
1. When you fix any of the tests, make sure to run all tests in the project to ensure nothing else is broken BEFORE YOU CLAIM THE FIX IS DONE.
1. We used to segregate the integration test codes for regular execution on TTY and non-TTY (GitHub) environments, and used `CI=true` to switch the behavior. This is no longer the case. All integration tests should be runnable using the CI-compatible mode in both environments without any environment variables. `CI` environment variable is no longer valid.
1. If you are a Generative AI engine, please use fewer number of commands. For example, do not change timeout parameter every time you run the test. VSCode will ask for the authorization for all variation of the command, and it will pause your workstream every time as I am not always available to authorize it on time. Using the same command with the same parameters will avoid your waiting time and achieve the result faster. Currently available commands are in `.claude/settings.local.json` (this is a live doc so please check the latest commands).
1. Any intermediate files and functions generated in the tests should be named with `debug_` prefix and cleaned up after the tests are done.
