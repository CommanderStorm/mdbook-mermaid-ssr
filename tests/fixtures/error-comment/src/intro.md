# Introduction

This test book verifies that the `on-error = "comment"` configuration works correctly.

When a Mermaid diagram fails to render, instead of failing the build, the preprocessor should emit an HTML comment with error details.