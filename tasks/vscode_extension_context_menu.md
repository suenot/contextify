# Task: VS Code Extension - "Copy for Context"

## User Request
The user wants to create a VS Code extension that adds a "Copy for Context" option to the file explorer's context menu. This option should allow selected files to be "added as context". The user provided an image illustrating this feature.

## Interpretation
"Adding files as context" likely means processing them with the `contextify` tool to extract their structure and content, and then making this information available, most likely by copying it to the clipboard for use with AI models or documentation.

The `contextify` project is a Rust CLI tool. A VS Code extension is typically a separate project written in TypeScript/JavaScript. The extension will *call* the `contextify` CLI.

## Plan

### Phase 1: Prepare `contextify` (Rust CLI) - Completed
1.  **Modify `contextify` CLI arguments (`src/main.rs`)**: Done.
    *   Added new option `--input-paths <path1[,path2,...]>`.
    *   Changed `--output` (`-o`) option to be optional for `stdout` output.
2.  **Refine `save_project_structure_and_files` function (`src/lib.rs`)**: Done.
    *   Updated function signature: `paths_to_process: &[PathBuf]`, `writer: &mut dyn Write`, `output_file_to_exclude: Option<&PathBuf>`.
    *   Adapted file collection for multiple inputs (files/directories).
    *   Display paths are now relative to CWD or absolute.
    *   Test handlers adapted to new signature (stubbed for `handle_custom_patterns_test` which needs careful content generation).
    *   Unit tests in `lib.rs` adapted to new signature and buffer-based output checking.

### Phase 2: Design VS Code Extension (Conceptual Guide)
*(This part is a guide for creating the extension, as the AI cannot directly create the extension project files in a separate workspace.)*

1.  **Project Setup (TypeScript)**:
    *   Use `yo code` to scaffold a new VS Code extension.
2.  **`package.json` Manifest**:
    *   Define a command, e.g., `contextify.copyForContext`.
    *   Contribute this command to the `explorer/context` menu group (`"when": "explorerResourceIsFolder || explorerResourceIsFile"`).
    *   Specify the display name "Copy for Context".
3.  **Extension Logic (`extension.ts`)**:
    *   Register the command handler.
    *   Inside the handler:
        *   Access the selected resource(s) (URI(s)) from the command arguments. For multiple selections, it will be an array of URIs.
        *   Convert URIs to absolute file system paths.
        *   Construct the `contextify` CLI command: `contextify --input-paths <abs_path1>,<abs_path2> ...` (note: no `--output` for stdout).
        *   Execute the `contextify` command using Node.js `child_process.exec` or `spawn`.
        *   Capture the `stdout` from `contextify`.
        *   Copy the captured output to the clipboard (`vscode.env.clipboard.writeText(output)`).
        *   Show a notification to the user (e.g., "Content copied to clipboard for N files.").

## Logical Reasoning & Checked Paths
*   Current `contextify` (`src/main.rs`, `src/lib.rs`) processes a single root directory (implicitly CWD). This needs generalization.
*   The `clap` crate in `src/main.rs` will be used for new/modified arguments.
*   Core processing is in `save_project_structure_and_files` in `src/lib.rs`.
*   A VS Code extension is a separate entity. This plan prepares `contextify` to be callable by such an extension.
*   The image provided confirms the desired UI feature.
*   Path handling for display and filtering needs to be consistent. Paths relative to CWD are a good default.
*   Exclusion of the output file itself from processing is important if output is a file.

## Open Questions for User (Follow-up if needed):
1.  When multiple files/folders are selected, should `contextify` produce a single combined structure text? (Current plan assumes yes, similar to how it processes a single root).
2.  Is copying to the clipboard the sole desired action for "adding to context"? (Current plan assumes yes).
3.  Does the `contextify` output format need to change (e.g., to JSON) for easier parsing by the extension, or is the current text format sufficient? (Current plan assumes current format is fine).

## Paths Already Checked/Considered:
*   `src/main.rs`: argument parsing, main application flow.
*   `src/lib.rs`: core file processing logic, statistics, existing test handlers.
*   `README.md`: for understanding current CLI usage.

## Paths to Revisit/Verify during implementation:
*   Interaction of new pathing logic with blacklist/whitelist patterns.
*   Robustness of `path.strip_prefix(&cwd)` for various path structures.
*   Ensuring existing tests in `lib.rs` still pass or are adapted after signature changes to `save_project_structure_and_files`.

## Next Steps (AI):
1.  ~~Modify `src/main.rs` for new CLI arguments and stdout capability.~~ (Done)
2.  ~~Modify `src/lib.rs` for new `save_project_structure_and_files` signature and logic.~~ (Done)
3.  ~~Update this task file with progress.~~ (Done)
4.  Await user feedback or next instructions regarding Phase 2 (VS Code extension development itself) or further refinements to `contextify`. 