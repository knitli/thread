use std::path::{Path, PathBuf, Component};

fn normalize_components(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                match components.last() {
                    Some(Component::RootDir) | Some(Component::Prefix(_)) => {
                        // Do not pop RootDir or Prefix to prevent path traversal outside root
                    }
                    Some(Component::ParentDir) | None => {
                        components.push(component);
                    }
                    _ => {
                        components.pop();
                    }
                }
            }
            Component::CurDir => {}
            _ => components.push(component),
        }
    }
    components.iter().collect()
}

fn main() {
    println!("{:?}", normalize_components(Path::new("/var/www/../../../../etc/passwd")));
}
