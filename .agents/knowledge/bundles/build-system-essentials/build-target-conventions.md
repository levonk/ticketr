---
type: Practice
title: Build Target Conventions
description: Mandatory, preferred, and potential Makefile targets — clean, build, test, lint, format, deploy, and more — with their responsibilities and naming conventions.
tags: [build-system, makefile, targets, conventions, build, test, lint, deploy]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-18"
  last-used: "2026-07-18"
---

# Build Target Conventions

Standardized Makefile targets ensure that any developer or AI agent can
interact with any project using the same command vocabulary. Every `Makefile`
across all projects implements a common set of targets, with preferred and
potential targets extending the vocabulary as needed.

## Mandatory Makefile Targets

The following targets *must* be present in every `Makefile` and execute corresponding scripts in the `/bin` directory:

*   **`clean`:**  Removes generated files (object files, executables, temporary files, etc.).
*   **`archive`:** Creates an archive of the project (e.g., a `7z` file).  This assumes the existence of an archiving script in `/bin`.
*   **`all`:** Builds the entire project.  This is often the default target, but can also be used for a specific configuration, defaulting to dev (development).
*   **`help` aka `usage`:** Displays a list of available targets with brief descriptions.  (See auto-generation details below).
*   **`check`:** Runs static analysis tools and other checks to ensure code quality and consistency.
*   **`format`:**  Formats the code according to project coding standards.
*   **`test`:**  Runs all unit tests.
*   **`lint`:**  Runs all linting tools to identify potential code issues. There should be individual targets for each type of lint that the main
    *  **lint-markdown:** Linting for `*.md` documentation and files.
*   **`coverage`:** Generates code coverage reports.
*   **`version`:** Displays the current project version.
*   **`status`:** Shows the current status of the project (e.g., Git branch, commit hash, build status).
*   **`watch`:**  Monitors files for changes and automatically rebuilds or reruns tests.
*   **`env`:** Sets up or displays the project's development environment.
*   **`docs`:**  Regenerates documentation from source code.
*   **`sync`:** Performs a Git rebase to synchronize the local branch with the remote.

## Preferred Makefile Targets

These targets may ALSO be aliased versions of targets that are specific to the project.

*   **`promote`:** Updates the build number for the next build.
*   **`new-module`:** Creates a new module or component within the project.  (Requires a well-defined directory structure).
*   **`run`:**  Executes the compiled program or application.
*   **`install`:**  Installs the project to a system directory.
*   **`uninstall`:** Removes the installed project from the system.
*   **`deploy`:** Deploys the project to a specified environment.  Should have variants for `dev` (default) and `prod`.  Example: `deploy-dev`, `deploy-prod`.

## Potential Targets

*   **`mutation`:** Performs mutation testing.
*   **`performance`:** Performs performance testing.
*   **`security`:** Performs security checks.
*   **`docker`:** builds release into docker package. This may be the standard build or release procedure if there isn't a compile step.
*   **`docker-run`:** runs a `docker-compose` or `docker run` for the generated package.
*  **`lint-make`:** Linting for `Makefiles` using multiple linters. This target runs:
    * **`lint-make-checkmake`:** Uses [checkmake](https://github.com/checkmake/checkmake) to lint Makefiles for common errors and best practices.
    * **`lint-make-bake`:** Uses [bake](https://github.com/EbodShojaei/bake) to check Makefile structure and patterns.
*  **`lint-ansible`:** Only if Ansible collections exist for the project.
*  **`lint-yaml`:** Only if yaml files exist for the project.

## .PHONY Declaration

All non-file targets must be declared as `.PHONY`:

```makefile
.PHONY: clean archive all help check format test lint run install uninstall coverage deploy release version status watch env docs sync new-module deploy-dev deploy-prod
```

## Default Goal

Set a default goal (usually `help` or `all`) so that running `make` without
arguments performs a useful action. Whatever the default is, in the case of it
being a default run of `make` with no arguments, it should follow calling
`help` and explaining which target is being called subsequently.

```makefile
.DEFAULT_GOAL := help
```

## Auto-Generated help Target

Implement a `help` (and have a `usage` alias) target that automatically
extracts target descriptions from the `Makefile`.

```makefile
help: ## Show this help message
	@grep -E '^[a-zA-Z_-]+:.*?## ' Makefile | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'
```

*   **Important:** Use `##` after the target definition to add the description.

```makefile
clean: ## Remove generated files
	$(BIN_DIR)/clean
```

## Sources

- Migrated from src/current/rules/software-dev/platforms/build-sys/makefile-essentials.md
