## 2025-03-09 - Insecure Path Normalization Logic
**Vulnerability:** `Component::ParentDir` incorrectly handled root/prefix directories and ignored relative traversals when the component list was empty or trailing `..`, enabling path traversal and incorrect relative path preservation.
**Learning:** Manually normalizing paths by just popping components on `..` fails to account for empty queues, existing `..` elements, and non-removable path prefixes/roots.
**Prevention:** Explicitly block `Component::ParentDir` from popping `Component::RootDir` or `Component::Prefix` and push `Component::ParentDir` when the list is empty or the last element is already `Component::ParentDir`.
