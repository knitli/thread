## 2024-04-17 - [Path Traversal]
**Vulnerability:** manual path resolution uses blind .pop()
**Learning:** To prevent path traversal vulnerabilities when manually normalizing paths with std::path::Component (e.g., during module path resolution in crates/flow), explicitly block Component::ParentDir from popping Component::RootDir or Component::Prefix. If the components list is empty or its last element is Component::ParentDir, the new Component::ParentDir must be pushed rather than ignored to correctly preserve relative paths like ../../a.
**Prevention:** Be careful with .pop() for ParentDir
