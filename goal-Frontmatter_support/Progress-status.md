# Progress Status

## update 01
- Implemented Phase 1 (Foundational - Parsing & Data Structures).
- Added frontmatter parsing module with YAML/TOML detection, conversion, and fixtures-based unit tests.
- Integrated parser into CLI flow to preserve existing frontmatter when rendering output.
- Next: begin Phase 2 by adding CLI surface for `frontmatter get` once foundational parsing stabilizes.
## update 02
- Delivered Phase 2 (Read-Only Operations) by introducing the `md-splice frontmatter get` command with support for dot/array key paths and configurable output formats (string/json/yaml).
- Added CLI wiring, parsing helpers, and stdout rendering utilities to expose parsed frontmatter without altering the Markdown body flow.
- Implemented comprehensive integration coverage for frontmatter reads and refreshed CLI help snapshot to surface the new subcommand.
- Next: move into Phase 3 by building `frontmatter set`/`delete`, including mutation helpers and serialization back to disk.
