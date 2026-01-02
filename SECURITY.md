# Security Policy

## Supported Versions

We release patches for security vulnerabilities. Currently supported versions:

| Version | Supported          |
| ------- | ------------------ |
| 0.3.0   | :white_check_mark: |

## Reporting a Vulnerability

We take the security of `mdbook-mermaid-ssr` seriously. If you believe you have found a security vulnerability, please report it to us as described below.

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report them privately using GitHub Security Advisories:

**[Report a vulnerability](https://github.com/CommanderStorm/mdbook-mermaid-ssr/security/advisories/new)**

You should receive a response between a few hours and 30 days, depending on if I am on holiday or not.
If for some reason you do not, please follow up by creating another advisory or contacting the maintainers.

Please include the following information in your report:

- Type of issue (e.g., buffer overflow, injection, denial of service, etc.)
- Full paths of source file(s) related to the manifestation of the issue
- The location of the affected source code (tag/branch/commit or direct URL)
- Any special configuration required to reproduce the issue
- Step-by-step instructions to reproduce the issue
- Proof-of-concept or exploit code (if possible)
- Impact of the issue, including how an attacker might exploit it

This information will help us triage your report more quickly.

## Security Considerations

### Chrome/Chromium Dependency

This project uses headless Chrome/Chromium via the `headless_chrome` crate to render Mermaid diagrams. Users should be aware of the following:

1. **System Requirements**: Chrome or Chromium must be installed on the build system
2. **Browser Security**: We rely on the security of the Chrome/Chromium browser itself
3. **Sandboxing**: The headless browser runs with default sandbox settings
4. **Network Access**: The browser should not require network access during diagram rendering, as all assets are bundled

### Input Handling

1. **Mermaid Code Blocks**: User-provided Mermaid syntax is processed by the Mermaid.js library in a headless browser environment
2. **Untrusted Input**: If building documentation from untrusted sources, be aware that malicious Mermaid syntax could potentially exploit vulnerabilities in Mermaid.js or the rendering engine
3. **Build Environment**: Only build documentation from trusted sources in production environments

### Dependencies

This project depends on several crates and the bundled Mermaid.js library:

1. **Rust Dependencies**: Managed via Cargo and regularly updated
2. **Mermaid.js**: Automatically updated via Dependabot when new versions are released
3. **Supply Chain**: We use `cargo-deny` (see `deny.toml`) for dependency policy enforcement (e.g., allowed licenses, banned crates, and sources), and `cargo-audit` to check for known vulnerabilities in Rust dependencies

Users can audit dependencies by running:
```bash
cargo tree
```

### Recommended Security Practices

When using `mdbook-mermaid-ssr`:

1. **Trusted Sources**: Only process Mermaid diagrams from trusted sources
2. **Isolated Builds**: Consider running builds in isolated or containerized environments
3. **Regular Updates**: Keep `mdbook-mermaid-ssr` and its dependencies up to date
4. **Chrome Updates**: Ensure Chrome/Chromium is kept up to date on build systems
5. **CI/CD Security**: When using in CI/CD pipelines, follow your platform's security best practices

### Known Limitations

1. **Headless Browser**: The security posture is partially dependent on the headless Chrome/Chromium installation
2. **Build-Time Execution**: Diagram rendering occurs at build time, not runtime, which limits the attack surface for end users
3. **No Network Isolation**: The preprocessor does not currently enforce network isolation for the headless browser

## Security Updates

Security updates will be released as patch versions and announced through:

- GitHub Security Advisories
- Release notes in CHANGELOG.md
- GitHub Releases page

## Disclosure Policy

When we receive a security bug report, we will:

1. Confirm the problem and determine affected versions
2. Audit code to find any similar problems
3. Prepare fixes for all supported versions
4. Release new versions as quickly as possible

## Comments on this Policy

If you have suggestions on how this process could be improved, please submit a pull request or open an issue.

## Attribution

This security policy is adapted from best practices in the Rust community and follows responsible disclosure guidelines.
