use crate::core::export::export::Export;
use crate::core::path::path::path::Path;
use std::any::Any;
use std::collections::HashMap;

pub mod export {
    use crate::core::path::path::path::Path;
    use std::any::Any;
    use std::collections::HashMap;

    /// Abstraction for the result of local computation.
    /// It is an AST decorated with the computation value.
    #[derive(Debug)]
    pub struct Export {
        pub(crate) map: HashMap<Path, Box<dyn Any>>,
    }

    #[macro_export]
    macro_rules! export {
        ($($x:expr),*) => {{
            let mut temp_map = HashMap::new();
            $(
                temp_map.insert($x.0, Box::new($x.1) as Box<dyn Any>);
            )*
            Export { map: temp_map }
        }};
    }
}

impl Export {
    /// Create new Export.
    ///
    /// # Returns
    ///
    /// The new Export.
    pub fn new() -> Self {
        Self {
            map: HashMap::new()
        }
    }

    /// Inserts a value in the Export at the given Path.
    ///
    /// # Arguments
    ///
    /// * `path` - The Path where to insert the value.
    /// * `value` - The value to insert.
    ///
    /// # Generic Parameters
    ///
    /// * `A` - The type of the value to insert. It must have a `'static` lifetime.
    /// * `F` - The type of the function to insert.
    ///
    /// # Returns
    ///
    /// The inserted value.
    pub fn put<A: 'static, F>(&mut self, path: Path, value: F) -> A
    where F: Fn() -> A {
        self.map.insert(path, Box::new(value()));
        value()
    }

    /// Returns the value at the given Path.
    ///
    /// # Arguments
    ///
    /// * `path` - The Path where to get the value.
    ///
    /// # Generic Parameters
    ///
    /// * `A` - The type of the value to get  to return. It must have a `'static` lifetime.
    ///
    /// # Returns
    ///
    /// The value at the given Path.
    pub fn get<A: 'static>(&self, path: &Path) -> Option<&A> {
        self.map
            .get(path)
            .and_then(|value| value.downcast_ref::<A>())
    }

    /// Obtain the root value.
    ///
    /// # Generic Parameters
    ///
    /// * `A` - The type of the value to return. It must have a `'static` lifetime.
    ///
    /// # Returns
    ///
    /// The root value.
    pub fn root<A: 'static>(&self) -> &A {
        self.get(&Path::new()).unwrap()
    }

    /// Returns the HashMap of the Export.
    ///
    /// # Returns
    ///
    /// The HashMap of the Export.
    pub fn paths(&self) -> &HashMap<Path, Box<dyn Any>> {
        &self.map
    }
}

impl From<HashMap<Path, Box<dyn Any>>> for Export {
    fn from(map: HashMap<Path, Box<dyn Any>>) -> Self {
        Self {
            map
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::path::slot::slot::Slot::{Nbr, Rep};

    #[test]
    fn test_new_empty() {
        let export: Export = Export::from(HashMap::new());
        assert!(export.map.is_empty())
    }

    #[test]
    fn test_new() {
        let mut map: HashMap<Path, Box<dyn Any>> = HashMap::new();
        map.insert(Path::from(vec![Rep(0), Nbr(0)]), Box::new(10));
        let export = Export::from(map);
        assert_eq!(export.map.len(), 1);
    }

    #[test]
    fn test_put() {
        let mut map: HashMap<Path, Box<dyn Any>> = HashMap::new();
        map.insert(Path::from(vec![Rep(0)]), Box::new(10));
        let mut export = Export::from(map);
        export.put(Path::from(vec![Rep(0), Nbr(0)]), || 20);
        export.put(Path::from(vec![Nbr(0)]), || "foo");
        assert_eq!(export.paths().len(), 3);
    }

    #[test]
    fn test_get() {
        let mut map: HashMap<Path, Box<dyn Any>> = HashMap::new();
        map.insert(Path::from(vec![Rep(0), Nbr(0)]), Box::new(10));
        let export = Export::from(map);
        assert_eq!(
            export.get::<i32>(&Path::from(vec![Rep(0), Nbr(0)])).unwrap(),
            &10
        );
    }

    #[test]
    fn test_get_none() {
        let mut map: HashMap<Path, Box<dyn Any>> = HashMap::new();
        map.insert(Path::from(vec![Rep(0), Nbr(0)]), Box::new(10));
        let export = Export::from(map);
        assert_eq!(export.get::<String>(&Path::from(vec![Rep(0), Nbr(0)])), None);
    }

    #[test]
    fn test_root() {
        let mut map: HashMap<Path, Box<dyn Any>> = HashMap::new();
        map.insert(Path::new(), Box::new(10));
        let export = Export::from(map);
        assert_eq!(export.root::<i32>(), &10);
    }

    #[test]
    #[should_panic]
    fn test_root_panic() {
        let mut map: HashMap<Path, Box<dyn Any>> = HashMap::new();
        map.insert(Path::new(), Box::new(10));
        let export = Export::from(map);
        assert_eq!(export.root::<String>(), &"foo");
    }

    #[test]
    fn test_paths() {
        let mut map: HashMap<Path, Box<dyn Any>> = HashMap::new();
        let mut map2: HashMap<Path, Box<dyn Any>> = HashMap::new();
        map.insert(Path::new(), Box::new(10));
        map2.insert(Path::new(), Box::new(10));
        let export = Export::from(map);
        assert!(export.map.keys().eq(map2.keys()));
    }

    #[test]
    fn test_empty_state() {
        let export: Export = Export::from(HashMap::new());
        let path = Path::from(vec![Nbr(0), Rep(0)]);
        assert_eq!(export.get::<i32>(&Path::new()), None);
        assert_eq!(export.get::<i32>(&path), None);
    }

    #[test]
    fn test_root_path() {
        let mut export: Export = Export::from(HashMap::new());
        export.put(Path::new(), ||String::from("foo"));
        assert_eq!(
            export.get::<String>(&Path::new()).unwrap(),
            export.root::<String>()
        );
        assert_eq!(
            export.get::<String>(&Path::new()),
            Some(&String::from("foo"))
        );
    }

    #[test]
    fn test_non_root_path() {
        let mut export: Export = Export::from(HashMap::new());
        let path = Path::from(vec![Nbr(0), Rep(0)]);
        export.put(path.clone(), || String::from("bar"));
        assert_eq!(export.get::<String>(&path).unwrap(), &String::from("bar"));
    }

    #[test]
    fn test_overwriting_with_different_type() {
        let mut export: Export = Export::from(HashMap::new());
        export.put(Path::new(), || String::from("foo"));
        assert_eq!(
            export.get::<String>(&Path::new()),
            Some(&String::from("foo"))
        );
        export.put(Path::new(), || 77);
        assert_eq!(export.get::<i32>(&Path::new()).unwrap(), &77);
    }
}
