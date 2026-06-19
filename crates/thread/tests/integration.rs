// SPDX-FileCopyrightText: 2026 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

#[cfg(all(feature = "ast", feature = "language"))]
#[test]
fn test_reexports_work() {
    use thread::language::{LanguageExt, Tsx};

    let ast = Tsx.ast_grep("const x = 1;");
    let matches: Vec<_> = ast.root().find_all("const $VAR = $VALUE").collect();
    assert_eq!(matches.len(), 1);
}

#[cfg(all(feature = "ast", feature = "services"))]
#[test]
fn test_service_reexports_work() {
    use thread::services::FileSystemContext;

    // Just check if we can use the types
    let _ctx = FileSystemContext::new(".").unwrap();
}
