use std::path::{Path, PathBuf, Component};

fn normalize_components(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                match components.last() {
                    Some(Component::RootDir) | Some(Component::Prefix(_)) => {
                        // Do not pop RootDir or Prefix
                    }
                    Some(Component::ParentDir) => {
                        components.push(component);
                    }
                    None => {
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
    components.into_iter().collect()
}

fn main() {
    println!("{:?}", normalize_components(Path::new("/var/www/../../../../etc/passwd")));
}
