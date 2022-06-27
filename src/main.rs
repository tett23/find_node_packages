extern crate serde_json;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::{env, path::Path, path::PathBuf};

#[derive(Serialize, Deserialize, Debug)]
struct Package {
    path: String,
    name: Option<String>,
    version: Option<String>,
}

fn main() {
    let arg = env::args();
    let path = arg.last().or(Some(
        env::current_dir().unwrap().to_string_lossy().to_string(),
    ));
    let path = path.expect("msg");
    let path = Path::new(&path);

    let ret = find_ancesters_node_modules(path)
        .iter()
        .flat_map(|path| find_node_modules(path.as_path()))
        .map(|item| to_package(item.as_path()))
        .flat_map(|item| match item {
            Some(item) => vec![item],
            None => Vec::new(),
        })
        .collect::<Vec<_>>();

    println!("{}", serde_json::to_string(&ret).unwrap());

    ()
}

fn find_ancesters_node_modules(path: &Path) -> Vec<PathBuf> {
    let node_modules = path.join("node_modules");

    let a = match node_modules.exists() {
        true => vec![node_modules],
        false => Vec::new(),
    };

    let parent = path.parent();
    match parent {
        Some(path) => [a, find_ancesters_node_modules(path)].concat(),
        None => a,
    }
}

fn find_node_modules(path: &Path) -> Vec<PathBuf> {
    let current = match has_package_json(path) {
        true => vec![path.to_path_buf()],
        false => Vec::new(),
    };
    let child_items = children_paths(path)
        .iter()
        .flat_map(|item| find_node_modules(item.as_path()))
        .filter(|item| has_package_json(item.as_path()))
        .collect::<Vec<_>>();

    [current, child_items].concat()
}

fn children_paths(path: &Path) -> Vec<PathBuf> {
    if !path.is_dir() {
        return Vec::new();
    }

    let readdir = path.read_dir().expect("msg");

    readdir
        .map(|item| item.expect("").path())
        .collect::<Vec<_>>()
}

fn has_package_json(path: &Path) -> bool {
    path.join("package.json").exists()
}

fn to_package(path: &Path) -> Option<Package> {
    let json = fs::read_to_string(path.join("package.json").as_path()).expect("msg");
    let package_json: Result<HashMap<String, Value>, _> = serde_json::from_str(&json);
    if package_json.is_err() {
        return None;
    }
    let package_json = package_json.unwrap();

    let version = match package_json.get("version") {
        Some(Value::String(v)) => Some(v.clone()),
        _ => None,
    };
    let name = match package_json.get("name") {
        Some(Value::String(v)) => Some(v.clone()),
        _ => None,
    };

    Some(Package {
        path: path.to_str().unwrap().to_string(),
        name: name,
        version: version,
    })
}
