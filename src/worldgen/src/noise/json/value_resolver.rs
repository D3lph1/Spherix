use anyhow::anyhow;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufReader, ErrorKind};
use std::path::PathBuf;

pub trait ValueResolver {
    fn resolve(&self, name: String) -> anyhow::Result<Value, Error>;
}

#[derive(Debug)]
pub enum Error {
    NotFound,
    Other(anyhow::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NotFound => {
                write!(f, "not found")
            }
            Error::Other(err) => {
                write!(f, "{}", err)
            }
        }
    }
}

impl std::error::Error for Error {}

pub struct NoReturnValueResolver;

impl ValueResolver for NoReturnValueResolver {
    fn resolve(&self, name: String) -> anyhow::Result<Value, Error> {
        unreachable!("NoReturnValueResolver::resolve() must not be ever called, but it was with the argument: {}", name)
    }
}

pub struct FilesystemValueResolver {
    root: PathBuf,
}

impl FilesystemValueResolver {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
        }
    }
}

impl ValueResolver for FilesystemValueResolver {
    fn resolve(&self, name: String) -> anyhow::Result<Value, Error> {
        let sub_path = name.strip_prefix("minecraft:").unwrap();
        let path = self.root.join(PathBuf::from(sub_path.to_owned() + ".json"));
        let file = File::open(path);

        match file {
            Ok(file) => {
                Ok(serde_json::from_reader(BufReader::new(file)).map_err(|e| Error::Other(anyhow!(e)))?)
            }
            Err(err) => {
                match err.kind() {
                    ErrorKind::NotFound => {
                        Err(Error::NotFound)
                    }
                    err => {
                        Err(Error::Other(anyhow!("Error during resolving \"{}\" value: {}", name, err)))
                    }
                }
            }
        }
    }
}

pub struct CascadeValueResolver {
    resolvers: Vec<Box<dyn ValueResolver>>,
}

impl CascadeValueResolver {
    pub fn new(resolvers: Vec<Box<dyn ValueResolver>>) -> Self {
        if resolvers.len() == 0 {
            panic!("It is required to specify at least one resolver")
        }

        Self {
            resolvers,
        }
    }
}

impl ValueResolver for CascadeValueResolver {
    fn resolve(&self, name: String) -> anyhow::Result<Value, Error> {
        for resolver in self.resolvers.iter() {
            let res = resolver.resolve(name.clone());

            if res.is_ok() {
                return res;
            }

            match res.as_ref().err().unwrap() {
                Error::NotFound => {
                    continue;
                }
                Error::Other(_) => {
                    return res;
                }
            }
        }

        Err(
            Error::Other(
                anyhow!(
                    "CascadeValueResolver failed to resolve \"{}\" value with all child resolvers: not found",
                    name
                )
            )
        )
    }
}

pub struct CachedValueResolver<V: ValueResolver> {
    inner: V,
    cache: RefCell<HashMap<String, Value>>,
}

impl<V: ValueResolver> CachedValueResolver<V> {
    pub fn new(inner: V) -> Self {
        Self {
            inner,
            cache: RefCell::new(Default::default()),
        }
    }
}

impl<V: ValueResolver> ValueResolver for CachedValueResolver<V> {
    fn resolve(&self, name: String) -> anyhow::Result<Value, Error> {
        if self.cache.borrow().contains_key(&name) {
            Ok(self.cache.borrow().get(&name).unwrap().clone())
        } else {
            let value = self.inner.resolve(name.clone())?;
            self.cache.borrow_mut().insert(name, value.clone());

            Ok(value)
        }
    }
}

pub struct MockValueResolver {
    map: HashMap<String, Value>,
}

impl MockValueResolver {
    pub fn new(map: HashMap<String, Value>) -> Self {
        Self {
            map,
        }
    }
}

impl ValueResolver for MockValueResolver {
    fn resolve(&self, name: String) -> anyhow::Result<Value, Error> {
        if self.map.contains_key(&name) {
            Ok(self.map.get(&name).unwrap().clone())
        } else {
            Err(Error::NotFound)
        }
    }
}
