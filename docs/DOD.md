# Defintion of Done

The product owner and developer agree to consider the work is done when the following conditions are met:

1. Gherkin feature files are updated with the new functionality.
1. Unit tests are written for all major functions and they are passing
1. The integration tests are updated based on the update in the feature file and they are passing.
1. The code is formatted using `cargo fmt` and passes `cargo clippy` with strict warnings.
1. The `docs/COMMANDS.md` is updated with the new functionality.
1. The `docs/ARCHITECTURE.md` is updated with the new components and their relationships if they are added/changed.
1. The `README.md` is updated as needed.
1. A pull request is created with a link to the commits that contains the changes.
1. The code is merged into the develop branch by a pull request and is ready for release.
1. The CI pipeline is passing without any errors.
1. The issue is In Review or Close in the GitHub Kanban board.
