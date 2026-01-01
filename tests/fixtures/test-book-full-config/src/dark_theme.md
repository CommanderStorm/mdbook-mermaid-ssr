# Dark Theme Diagram

This chapter demonstrates the dark theme configuration.

```mermaid
graph TB
    A[Dark Theme] --> B[Configuration]
    B --> C{Applied?}
    C -->|Yes| D[Success]
    C -->|No| E[Error]
    D --> F[Visible in SVG]
    style A fill:#2d3748,stroke:#4a5568,color:#fff
    style B fill:#2d3748,stroke:#4a5568,color:#fff
    style C fill:#2d3748,stroke:#4a5568,color:#fff
    style D fill:#48bb78,stroke:#38a169,color:#fff
    style E fill:#f56565,stroke:#e53e3e,color:#fff
    style F fill:#4299e1,stroke:#3182ce,color:#fff
```

The diagram above should use dark color schemes. When inspecting the SVG output, you should see dark colors applied to the diagram elements based on the `theme = "dark"` configuration.

## Flowchart Example

```mermaid
flowchart LR
    Start --> Process
    Process --> Decision{Check}
    Decision -->|Pass| Success
    Decision -->|Fail| Retry
    Retry --> Process
    Success --> End
```

This flowchart should also reflect the dark theme colors in the generated SVG.