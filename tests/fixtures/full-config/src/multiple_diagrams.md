# Multiple Diagrams

This chapter tests multiple Mermaid diagrams on a single page, all rendered with the combined configuration settings.

## Diagram 1: Flowchart

```mermaid
flowchart TD
    Start([Start Process]) --> Init[Initialize]
    Init --> Check{Valid?}
    Check -->|Yes| Process[Process Data]
    Check -->|No| Error[Handle Error]
    Process --> Save[Save Results]
    Error --> Log[Log Error]
    Save --> End([End])
    Log --> End
```

## Diagram 2: Sequence Diagram

```mermaid
sequenceDiagram
    autonumber
    Client->>+Server: Authentication Request
    Server->>+Database: Verify Credentials
    Database-->>-Server: Credentials Valid
    Server-->>-Client: Auth Token
    Client->>+Server: API Request with Token
    Server->>+Cache: Check Cache
    Cache-->>-Server: Cache Miss
    Server->>+Database: Fetch Data
    Database-->>-Server: Data Response
    Server-->>-Client: API Response
```

## Diagram 3: Class Diagram

```mermaid
classDiagram
    class Animal {
        +String name
        +int age
        +makeSound()
    }
    class Dog {
        +String breed
        +bark()
    }
    class Cat {
        +String color
        +meow()
    }
    Animal <|-- Dog
    Animal <|-- Cat
```

## Diagram 4: Entity Relationship

```mermaid
erDiagram
    CUSTOMER ||--o{ ORDER : places
    ORDER ||--|{ LINE-ITEM : contains
    CUSTOMER {
        string name
        string email
        string phone
    }
    ORDER {
        int orderNumber
        date orderDate
        string status
    }
    LINE-ITEM {
        int quantity
        decimal price
    }
    PRODUCT ||--o{ LINE-ITEM : includes
    PRODUCT {
        string name
        string description
        decimal price
    }
```

## Diagram 5: Gantt Chart

```mermaid
gantt
    title Project Timeline
    dateFormat YYYY-MM-DD
    section Planning
    Requirements    :a1, 2024-01-01, 7d
    Design         :a2, after a1, 10d
    section Development
    Backend        :b1, after a2, 20d
    Frontend       :b2, after a2, 25d
    Integration    :b3, after b1, 7d
    section Testing
    Unit Tests     :c1, after b3, 5d
    Integration Tests :c2, after b3, 7d
    UAT           :c3, after c2, 10d
```

All five diagrams above should render with:
- Dark theme colors
- Hand-drawn (sketchy) style
- Loose security level
- 45-second timeout
- Error comments (if any fail)

This demonstrates that the configuration is consistently applied across all diagram types on a single page.