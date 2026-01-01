# Hand-Drawn Style

This chapter demonstrates the hand-drawn (sketchy) look configuration.

```mermaid
graph TD
    A[Hand-Drawn Style] --> B[Sketchy Lines]
    B --> C[Rough Edges]
    C --> D[Artistic Look]
    D --> E[More Human Feel]
```

The diagram above should have a sketchy, hand-drawn appearance with rough edges and imperfect lines, thanks to the `look = "handDrawn"` configuration.

## Sequence Diagram with Hand-Drawn Style

```mermaid
sequenceDiagram
    participant User
    participant System
    participant Database
    
    User->>System: Request Data
    activate System
    System->>Database: Query
    activate Database
    Database-->>System: Results
    deactivate Database
    System-->>User: Response
    deactivate System
```

This sequence diagram should also exhibit the hand-drawn aesthetic with sketchy lines and a more organic, less rigid appearance.

## State Diagram

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Processing: Start
    Processing --> Success: Complete
    Processing --> Failed: Error
    Success --> [*]
    Failed --> Retry
    Retry --> Processing
```

Even state diagrams should show the hand-drawn style, making the entire book's diagrams consistent with this artistic approach.