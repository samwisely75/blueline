# Development Workflow (The "Workflow")

Developers and Generative AI Engines like Claude Code should strictly follow this workflow to implement new features or fix bugs in the codebase. This process ensures that changes are made systematically, tested thoroughly, and documented properly.

1. Pick up the top-most item in the `Ready` state on [blueline GitHub Kanban](https://github.com/users/samwisely75/projects/1) by the following command. If the result is empty, ask for it.

    ```shell
    gh project item-list 1 \
         --owner samwisely75 \
         --format json \
         --limit 1000 \
         --jq '.items[] | select(.status == "Ready")'
    ```

1. Plan the implementation and todos. Ask for clarification if needed.
1. Prune the origin and fetch the latest changes from the `develop` branch:

    ```shell
    git fetch origin
    git checkout develop
    git pull origin develop
    ```

1. Create a new branch from the `develop` branch with a descriptive name, e.g., `feature/new-feature` or `bugfix/fix-issue-123`.
1. Move the Kanban item to the `In progress` state. The command is:

    ```shell
    gh project item-edit \
         --id $item_id \
         --field-id PVTSSF_lAHODQVrPs4A_FfHzgyS000 \
         --single-select-option-id 47fc9ee4 \
         --project-id PVT_kwHODQVrPs4A_FfH
    ```

1. Update the feature file to dictate the specification of the feature. If the feature is already implemented, update the existing test to reflect the new behavior.
1. Implement the changes.
1. Create or update comprehensive unit tests for the changes.
1. Run all tests including integration tests to ensure everything works as expected.
1. Make sure to leave comments following the coding guidelines. It is particularly important to leave comments when you change the code for a bug fix.
1. Run `/scripts/git-commit-precheck.sh` to see if the code passes pre-commit checks. If it fails, fix the issues and run the script again until it passes.
     - This script will run `cargo clippy --all-targets --all-features -- -D warnings` and `cargo fmt` to ensure code quality and formatting.
     - If you are using a Generative AI Engine like Claude Code, make sure to run this script before committing any changes.
1. Commit all changes with a clear and concise commit message. Do not leave out the code files you updated to fix clippy warnings and what `cargo fmt` modified.
1. Create a pull request (PR) with a clear and concise description of the changes made, including the issue number if applicable.
     - The PR title should be descriptive and follow the format `Fix #issue_number: Short description of the change`.
     - The PR description should include:
        - A summary of the changes made
        - The issue number(s) related to the changes
        - Any additional context or information that may be helpful for reviewers
1. Move the Kanban item to the `In Review` state. The command is:

    ```shell
    gh project item-edit \
         --id $item_id \
         --field-id PVTSSF_lAHODQVrPs4A_FfHzgyS000 \
         --single-select-option-id df73e18b \
         --project-id PVT_kwHODQVrPs4A_FfH
    ```

1. Address any feedback on PR and make necessary changes.
1. Increment the version number in `Cargo.toml`. If a new feature is added, increment the minor version. If a bug is fixed, increment the patch version. If a breaking change is made, increment the major version.
1. Update the changelog in `docs/CHANGELOG.md` with a summary of the changes made.
1. Create a git tag for the same version number with "v", e.g., `git tag v1.0.0`.
1. Go to step 1.
