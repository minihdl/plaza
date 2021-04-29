use internship;
use std::fmt;

#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Var {
    pub name: internship::Intern<str>,
}

impl Var {
    pub fn len(&self) -> usize { self.name.len() }
}

impl Clone for Var {
    fn clone(&self) -> Var {
        Var {
            name: internship::intern(&*self.name),
        }
    }
}

impl From<&str> for Var {
    fn from(name: &str) -> Var {
        Var {
            name: internship::intern(name),
        }
    }
}

impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &*self.name)
    }
}
