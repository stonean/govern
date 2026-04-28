# Initialize

Post-copy setup for projects created from this one via `/{project}:create`. This command is called automatically by the create command after generic scaffolding is complete — it is not intended to be run directly.

## Purpose

The create command handles language-agnostic scaffolding: copying specs, commands, configuration, and implementation files, then renaming project references in markdown files. This command handles everything language-specific and project-specific that create cannot do generically, such as:

- Renaming module paths and import statements in source code
- Updating build configuration files with the new project name
- Regenerating lock files or checksums
- Any other transformations specific to this project's tech stack

## Context

This command runs in the new project directory. It receives the following inputs from the create command:

- `{slug}` — the new project slug
- `{display-name}` — the new project display name
- `{git-remote-url}` — the new project's git remote URL
- `{source}` — the source project name

## Steps

<!-- Fill in the language-specific and project-specific post-copy steps for your
     tech stack. The create command handles generic scaffolding (copying specs,
     commands, markdown files, renaming project references in markdown). This
     command handles everything create cannot do generically.

     Common steps to include:

     1. **Rename module/package paths** — update the module declaration and all
        import statements in source files to match the new project name
        (e.g., Go module path, Python package name, Node.js package.json name)

     2. **Update build configuration** — replace the source project name in
        build files like Makefile, Dockerfile, docker-compose.yml, CI config

     3. **Regenerate lock files** — run your package manager to update checksums
        (e.g., go mod tidy, npm install, bundle install)

     4. **Update project metadata** — any project-specific references in config
        files that the generic rename in create.md would miss (e.g., binary
        names, container image tags, service names)

     Delete this comment block and replace it with your concrete steps. See
     Anvil's initialize.md for a Go-specific example. -->
